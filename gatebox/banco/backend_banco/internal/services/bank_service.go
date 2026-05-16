package services

import (
	"context"
	"crypto/sha256"
	"database/sql"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"fmt"
	"log"
	"math/rand/v2"
	"regexp"
	"strings"
	"time"

	"banco_saczuck_backend/internal/gatebox_client"
	"banco_saczuck_backend/internal/models"
	"banco_saczuck_backend/internal/repositories"
)

type BankService struct {
	repo    *repositories.Repository
	gatebox *gatebox_client.Client
	log     *log.Logger
}

var onlyDigitsRegex = regexp.MustCompile(`\D`)

func NewBankService(repo *repositories.Repository, gatebox *gatebox_client.Client, lg *log.Logger) *BankService {
	if lg == nil {
		lg = log.Default()
	}
	return &BankService{repo: repo, gatebox: gatebox, log: lg}
}

func (s *BankService) CreateAccount(ctx context.Context, fullName, personType, document, email, password string) (models.Account, error) {
	normalizedType := strings.ToUpper(strings.TrimSpace(personType))
	normalizedDocument := normalizeDigits(document)
	if fullName == "" || email == "" || len(password) < 8 || !isValidDocument(normalizedType, normalizedDocument) {
		return models.Account{}, errors.New("invalid account payload")
	}
	hash := sha256.Sum256([]byte(password))
	userID := newID()
	account := models.Account{
		ID:            newID(),
		UserID:        userID,
		FullName:      fullName,
		PersonType:    normalizedType,
		Document:      normalizedDocument,
		Email:         strings.ToLower(email),
		Agency:        "0001",
		AccountNumber: fmt.Sprintf("100%06d", rand.IntN(999999)),
		PixKey:        "pix-" + userID[:12],
		Status:        "ACTIVE",
	}
	if err := s.repo.CreateAccount(ctx, account, hex.EncodeToString(hash[:])); err != nil {
		return models.Account{}, err
	}
	_ = s.repo.AppendAuditLog(ctx, account.ID, "ACCOUNT_CREATED", account.Email, "{}")
	return account, nil
}

func (s *BankService) Login(ctx context.Context, email, password string) (string, string, error) {
	userID, hash, err := s.repo.GetCredentialsByEmail(ctx, strings.ToLower(email))
	if err != nil {
		return "", "", errors.New("invalid credentials")
	}
	check := sha256.Sum256([]byte(password))
	if hex.EncodeToString(check[:]) != hash {
		return "", "", errors.New("invalid credentials")
	}
	token := base64.StdEncoding.EncodeToString([]byte(userID + ":" + fmt.Sprint(time.Now().Unix())))
	return token, userID, nil
}

func (s *BankService) GetMyAccount(ctx context.Context, userID string) (models.Account, error) {
	return s.repo.GetAccountByUserID(ctx, userID)
}

func (s *BankService) GetBalance(ctx context.Context, accountID string) (models.Balance, error) {
	return s.repo.GetBalance(ctx, accountID)
}

type TopupRequest struct {
	SourceAccountID       string
	AmountCents           int64
	EntryType             string
	Note                  string
	TargetAgency          string
	TargetAccountNumber   string
	TargetPersonType      string
	TargetDocument        string
}

func (s *BankService) Topup(ctx context.Context, req TopupRequest) error {
	amount := req.AmountCents
	if amount <= 0 {
		return errors.New("amount must be positive")
	}
	targetAccountID := req.SourceAccountID
	targetDetails := "self"
	if req.TargetAgency != "" || req.TargetAccountNumber != "" || req.TargetPersonType != "" || req.TargetDocument != "" {
		normalizedType := strings.ToUpper(strings.TrimSpace(req.TargetPersonType))
		normalizedDocument := normalizeDigits(req.TargetDocument)
		if req.TargetAgency == "" || req.TargetAccountNumber == "" || !isValidDocument(normalizedType, normalizedDocument) {
			return errors.New("invalid target account data")
		}
		target, err := s.repo.GetAccountByBankData(ctx, req.TargetAgency, req.TargetAccountNumber, normalizedType, normalizedDocument)
		if err != nil {
			if err == sql.ErrNoRows {
				return errors.New("target account not found")
			}
			return err
		}
		targetAccountID = target.ID
		targetDetails = fmt.Sprintf("%s/%s %s:%s", target.Agency, target.AccountNumber, target.PersonType, target.Document)
	}

	details := fmt.Sprintf("%s: %s | target=%s", req.EntryType, req.Note, targetDetails)
	if err := s.repo.AddTopup(ctx, targetAccountID, amount, details); err != nil {
		return err
	}
	return s.repo.AppendAuditLog(ctx, req.SourceAccountID, "TOPUP", "user", fmt.Sprintf(`{"amount_cents":%d,"type":"%s","target":"%s"}`, amount, req.EntryType, targetDetails))
}

func (s *BankService) ListTransactions(ctx context.Context, accountID string) ([]models.Transaction, error) {
	return s.repo.ListTransactions(ctx, accountID)
}

type PaymentRequest struct {
	AccountID       string
	Method          string
	Reference       string
	IdempotencyKey  string
	SimulationState string
}

func (s *BankService) Pay(ctx context.Context, req PaymentRequest) (repositories.PaymentRecord, error) {
	if req.IdempotencyKey == "" {
		return repositories.PaymentRecord{}, errors.New("missing idempotency key")
	}
	if existing, err := s.repo.FindPaymentByIdempotency(ctx, req.AccountID, req.IdempotencyKey); err == nil {
		return existing, nil
	} else if err != sql.ErrNoRows {
		return repositories.PaymentRecord{}, err
	}

	refTrim := strings.TrimSpace(req.Reference)
	s.log.Printf(
		"[PAY] begin method=%s account=%s idempotency_key=%s reference=%s",
		req.Method, req.AccountID, req.IdempotencyKey, referenceLogSummary(refTrim),
	)

	var validation gatebox_client.ChargeValidationResponse
	var statusCode int
	var logBody string
	var err error

	if v, sc, lb, ok := pixEmvGateboxStub(refTrim); ok {
		validation, statusCode, logBody = v, sc, lb
		s.log.Printf(
			"[PAY] pix_emv_stub account=%s amount_cents=%d charge_id=%s receiver=%s reference=%s",
			req.AccountID, validation.AmountCents, validation.ChargeID, validation.Receiver, referenceLogSummary(refTrim),
		)
	} else {
		validation, statusCode, logBody, err = s.gatebox.ValidateCharge(ctx, refTrim)
		if err != nil {
			s.log.Printf(
				"[PAY] gatebox_validate_error account=%s err=%v status=%d exchange_tail=%s",
				req.AccountID, err, statusCode, truncateForLog(logBody, 4000),
			)
			_ = s.repo.SaveGateboxLog(ctx, "", "VALIDATE_CHARGE_ERROR",
				truncateForLog(refTrim, 8000),
				truncateForLog(fmt.Sprintf("err=%v | %s", err, logBody), 12000),
				statusCode, 0)
			return repositories.PaymentRecord{}, fmt.Errorf("gatebox validate: %w (ref=%s)", err, referenceLogSummary(refTrim))
		}
		s.log.Printf(
			"[PAY] gatebox_validate_response account=%s http=%d valid=%v amount_cents=%d failure_msg=%q charge_id=%q ref=%s",
			req.AccountID, statusCode, validation.Valid, validation.AmountCents, validation.FailureMessage, validation.ChargeID, referenceLogSummary(refTrim),
		)
	}
	if statusCode >= 400 || !validation.Valid {
		detail := formatValidationForLog(statusCode, validation, logBody)
		s.log.Printf("[PAY] gatebox_charge_invalid account=%s detail=%s", req.AccountID, truncateForLog(detail, 6000))
		_ = s.repo.SaveGateboxLog(ctx, "", "VALIDATE_CHARGE_INVALID",
			truncateForLog(refTrim, 8000),
			truncateForLog(detail, 12000),
			statusCode, 0)
		msg := strings.TrimSpace(validation.FailureMessage)
		if msg == "" {
			msg = "validation failed"
		}
		return repositories.PaymentRecord{}, fmt.Errorf("gatebox charge invalid: %s (http=%d valid=%t ref=%s)", msg, statusCode, validation.Valid, referenceLogSummary(refTrim))
	}

	amountCents := clampAmountCentsForDB(validation.AmountCents)

	p := repositories.PaymentRecord{
		ID:             newID(),
		AccountID:      req.AccountID,
		GateboxCharge:  validation.ChargeID,
		Method:         req.Method,
		State:          models.PaymentValidated,
		AmountCents:    amountCents,
		IdempotencyKey: req.IdempotencyKey,
	}
	if err := s.repo.CreatePayment(ctx, p); err != nil {
		s.log.Printf(
			"[PAY] create_payment_failed account=%s payment_id=%s amount_cents=%d err=%v reference=%s",
			req.AccountID, p.ID, amountCents, err, referenceLogSummary(refTrim),
		)
		_ = s.repo.SaveGateboxLog(ctx, "", "CREATE_PAYMENT_FAILED",
			truncateForLog(refTrim, 8000),
			truncateForLog(fmt.Sprintf("payment_id=%s amount_cents=%d err=%v", p.ID, amountCents, err), 8000),
			0, 0)
		return repositories.PaymentRecord{}, fmt.Errorf("create payment: %w (amount_cents=%d ref=%s)", err, amountCents, referenceLogSummary(refTrim))
	}
	_ = s.repo.SaveGateboxLog(ctx, p.ID, "VALIDATE_CHARGE", truncateForLog(refTrim, 8000), truncateForLog(logBody, 12000), statusCode, 0)

	state, reason := s.resolveSimulation(req.SimulationState, req.AccountID, amountCents)
	_ = s.repo.UpdatePaymentStatus(ctx, p.ID, state, reason)
	_ = s.repo.AddTransactionForPayment(ctx, req.AccountID, p.AmountCents, string(state), p.GateboxCharge, p.ID, reason)
	_ = s.repo.AppendAuditLog(ctx, req.AccountID, "PAYMENT_"+string(state), "user", fmt.Sprintf(`{"payment_id":"%s"}`, p.ID))
	if err := s.gatebox.NotifyStatus(ctx, p.ID, p.GateboxCharge, string(state)); err != nil {
		s.log.Printf("[PAY] notify_status_nonfatal payment_id=%s charge_id=%s err=%v", p.ID, p.GateboxCharge, err)
	}
	s.log.Printf(
		"[PAY] done payment_id=%s account=%s state=%s amount_cents=%d method=%s reference=%s",
		p.ID, req.AccountID, state, p.AmountCents, req.Method, referenceLogSummary(refTrim),
	)
	p.State = state
	p.FailReason = reason
	// WhatsApp: notificação via Gatebox notify-status → apicash-whatsapp (evita duplicata).
	return p, nil
}

func (s *BankService) resolveSimulation(state, accountID string, amount int64) (models.PaymentState, string) {
	switch strings.ToUpper(state) {
	case "APPROVED":
		return models.PaymentApproved, "simulated_approved"
	case "PENDING":
		return models.PaymentPending, "simulated_pending"
	case "REJECTED":
		return models.PaymentRejected, "simulated_rejected"
	case "INSUFFICIENT_BALANCE":
		return models.PaymentFailed, "insufficient_balance"
	case "TIMEOUT":
		return models.PaymentTimeout, "bank_timeout"
	case "TEMP_FAILURE":
		return models.PaymentFailed, "temporary_failure"
	case "REFUNDED":
		return models.PaymentRefunded, "simulated_refund"
	default:
		return models.PaymentApproved, "auto_default"
	}
}

func newID() string {
	sum := sha256.Sum256([]byte(fmt.Sprintf("%d-%d", time.Now().UnixNano(), rand.Int())))
	return hex.EncodeToString(sum[:16])
}

func normalizeDigits(v string) string {
	return onlyDigitsRegex.ReplaceAllString(v, "")
}

func isValidDocument(personType, document string) bool {
	switch personType {
	case "PF":
		return len(document) == 11
	case "PJ":
		return len(document) == 14
	default:
		return false
	}
}

