package controllers

import (
	"encoding/json"
	"net/http"
	"strings"

	"banco_saczuck_backend/internal/models"
	"banco_saczuck_backend/internal/services"
)

type Controller struct {
	service *services.BankService
}

func New(service *services.BankService) *Controller {
	return &Controller{service: service}
}

func (c *Controller) CreateAccount(w http.ResponseWriter, r *http.Request) {
	var in struct {
		FullName   string `json:"full_name"`
		PersonType string `json:"person_type"`
		Document   string `json:"document"`
		Email      string `json:"email"`
		Password   string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
		writeErr(w, http.StatusBadRequest, err.Error())
		return
	}
	account, err := c.service.CreateAccount(r.Context(), in.FullName, in.PersonType, in.Document, in.Email, in.Password)
	if err != nil {
		writeErr(w, http.StatusBadRequest, err.Error())
		return
	}
	writeJSON(w, http.StatusCreated, account)
}

func (c *Controller) Login(w http.ResponseWriter, r *http.Request) {
	var in struct {
		Email    string `json:"email"`
		Password string `json:"password"`
	}
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
		writeErr(w, http.StatusBadRequest, err.Error())
		return
	}
	token, userID, err := c.service.Login(r.Context(), in.Email, in.Password)
	if err != nil {
		writeErr(w, http.StatusUnauthorized, err.Error())
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"access_token": token, "user_id": userID, "token_type": "Bearer"})
}

func (c *Controller) GetMe(w http.ResponseWriter, r *http.Request) {
	account, err := c.service.GetMyAccount(r.Context(), userID(r))
	if err != nil {
		writeErr(w, http.StatusNotFound, err.Error())
		return
	}
	writeJSON(w, http.StatusOK, account)
}

func (c *Controller) GetBalance(w http.ResponseWriter, r *http.Request) {
	account, err := c.service.GetMyAccount(r.Context(), userID(r))
	if err != nil {
		writeErr(w, http.StatusUnauthorized, "account not found")
		return
	}
	balance, err := c.service.GetBalance(r.Context(), account.ID)
	if err != nil {
		writeErr(w, http.StatusInternalServerError, err.Error())
		return
	}
	writeJSON(w, http.StatusOK, balance)
}

func (c *Controller) Topup(w http.ResponseWriter, r *http.Request) {
	account, err := c.service.GetMyAccount(r.Context(), userID(r))
	if err != nil {
		writeErr(w, http.StatusUnauthorized, "account not found")
		return
	}
	var in struct {
		AmountCents         int64  `json:"amount_cents"`
		EntryType           string `json:"entry_type"`
		Note                string `json:"note"`
		TargetAgency        string `json:"target_agency"`
		TargetAccountNumber string `json:"target_account_number"`
		TargetPersonType    string `json:"target_person_type"`
		TargetDocument      string `json:"target_document"`
	}
	if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
		writeErr(w, http.StatusBadRequest, err.Error())
		return
	}
	if err := c.service.Topup(r.Context(), services.TopupRequest{
		SourceAccountID:     account.ID,
		AmountCents:         in.AmountCents,
		EntryType:           in.EntryType,
		Note:                in.Note,
		TargetAgency:        in.TargetAgency,
		TargetAccountNumber: in.TargetAccountNumber,
		TargetPersonType:    in.TargetPersonType,
		TargetDocument:      in.TargetDocument,
	}); err != nil {
		writeErr(w, http.StatusBadRequest, err.Error())
		return
	}
	writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
}

func (c *Controller) ListTransactions(w http.ResponseWriter, r *http.Request) {
	account, err := c.service.GetMyAccount(r.Context(), userID(r))
	if err != nil {
		writeErr(w, http.StatusUnauthorized, "account not found")
		return
	}
	items, err := c.service.ListTransactions(r.Context(), account.ID)
	if err != nil {
		writeErr(w, http.StatusInternalServerError, err.Error())
		return
	}
	writeJSON(w, http.StatusOK, map[string]any{"items": items})
}

func (c *Controller) PayWithMethod(method string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		account, err := c.service.GetMyAccount(r.Context(), userID(r))
		if err != nil {
			writeErr(w, http.StatusUnauthorized, "account not found")
			return
		}
		var in struct {
			Reference       string `json:"reference"`
			SimulationState string `json:"simulation_state"`
		}
		if err := json.NewDecoder(r.Body).Decode(&in); err != nil {
			writeErr(w, http.StatusBadRequest, err.Error())
			return
		}
		idem := r.Header.Get("Idempotency-Key")
		p, err := c.service.Pay(r.Context(), services.PaymentRequest{
			AccountID:       account.ID,
			Method:          strings.ToUpper(method),
			Reference:       in.Reference,
			IdempotencyKey:  idem,
			SimulationState: in.SimulationState,
		})
		if err != nil {
			writeErr(w, http.StatusBadRequest, err.Error())
			return
		}
		writeJSON(w, http.StatusOK, p)
	}
}

func (c *Controller) UpdatePaymentStatus(state models.PaymentState) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		id := r.PathValue("id")
		writeJSON(w, http.StatusOK, map[string]string{"payment_id": id, "state": string(state)})
	}
}

func (c *Controller) Health(w http.ResponseWriter, _ *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"service": "banco_saczuck_backend", "status": "ok"})
}

func (c *Controller) GetSimulationSettings(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]any{
		"auto_approve":               false,
		"auto_reject":                false,
		"processing_delay_ms":        0,
		"random_failure_rate":        0,
		"timeout_enabled":            false,
		"insufficient_balance":       false,
		"webhook_active":             true,
		"gatebox_environment":        "sandbox",
		"supported_payment_statuses": []string{"APPROVED", "PENDING", "REJECTED", "FAILED", "TIMEOUT", "REFUNDED"},
		"supported_payment_methods":  []string{"PIX", "QRCODE", "LINK", "CARD", "VALIDATE"},
	})
}

func (c *Controller) PutSimulationSettings(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusOK, map[string]string{"status": "updated"})
}

func (c *Controller) ReceiveGateboxWebhook(w http.ResponseWriter, r *http.Request) {
	writeJSON(w, http.StatusAccepted, map[string]string{"status": "received"})
}

func writeErr(w http.ResponseWriter, status int, message string) {
	writeJSON(w, status, map[string]string{"error": message})
}

func writeJSON(w http.ResponseWriter, status int, payload any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(payload)
}

func userID(r *http.Request) string {
	if v := r.Header.Get("X-User-ID"); v != "" {
		return v
	}
	return "demo-user"
}

