package repositories

import (
	"context"
	"database/sql"
	"time"

	"banco_saczuck_backend/internal/models"
)

type PaymentRecord struct {
	ID             string             `json:"id"`
	AccountID      string             `json:"account_id"`
	GateboxCharge  string             `json:"gatebox_charge_id,omitempty"`
	Method         string             `json:"payment_method,omitempty"`
	State          models.PaymentState `json:"payment_state"`
	AmountCents    int64              `json:"amount_cents"`
	IdempotencyKey string             `json:"idempotency_key,omitempty"`
	FailReason     string             `json:"fail_reason,omitempty"`
}

func (r *Repository) CreatePayment(ctx context.Context, p PaymentRecord) error {
	now := time.Now().UTC().Format(time.RFC3339)
	_, err := r.db.ExecContext(ctx, `INSERT INTO banco_payments (id, account_id, gatebox_charge_id, payment_method, payment_state, amount_cents, idempotency_key, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`,
		p.ID, p.AccountID, p.GateboxCharge, p.Method, string(p.State), p.AmountCents, p.IdempotencyKey, now, now)
	return err
}

func (r *Repository) FindPaymentByIdempotency(ctx context.Context, accountID, key string) (PaymentRecord, error) {
	var p PaymentRecord
	err := r.db.QueryRowContext(ctx, `SELECT id, account_id, gatebox_charge_id, payment_method, payment_state, amount_cents, idempotency_key, COALESCE(fail_reason,'') FROM banco_payments WHERE account_id = $1 AND idempotency_key = $2`,
		accountID, key).Scan(&p.ID, &p.AccountID, &p.GateboxCharge, &p.Method, &p.State, &p.AmountCents, &p.IdempotencyKey, &p.FailReason)
	return p, err
}

func (r *Repository) UpdatePaymentStatus(ctx context.Context, paymentID string, state models.PaymentState, reason string) error {
	now := time.Now().UTC().Format(time.RFC3339)
	_, err := r.db.ExecContext(ctx, `UPDATE banco_payments SET payment_state = $1, fail_reason = $2, updated_at = $3 WHERE id = $4`, string(state), reason, now, paymentID)
	return err
}

func (r *Repository) AddTransactionForPayment(ctx context.Context, accountID string, amount int64, status, gateboxChargeID, paymentID, details string) error {
	now := time.Now().UTC().Format(time.RFC3339)
	_, err := r.db.ExecContext(ctx, `INSERT INTO banco_transactions (id, account_id, type, amount_cents, status, gatebox_charge_id, payment_id, details, created_at) VALUES (md5(random()::text || clock_timestamp()::text), $1, 'PAYMENT', $2, $3, $4, $5, $6, $7)`,
		accountID, amount, status, gateboxChargeID, paymentID, details, now)
	return err
}

func (r *Repository) ListTransactions(ctx context.Context, accountID string) ([]models.Transaction, error) {
	rows, err := r.db.QueryContext(ctx, `SELECT id, type, amount_cents, status, COALESCE(payer,''), COALESCE(receiver,''), COALESCE(gatebox_charge_id,''), COALESCE(payment_id,''), COALESCE(details,''), created_at FROM banco_transactions WHERE account_id = $1 ORDER BY created_at DESC`, accountID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []models.Transaction
	for rows.Next() {
		var t models.Transaction
		if err := rows.Scan(&t.ID, &t.Type, &t.AmountCents, &t.Status, &t.Payer, &t.Receiver, &t.GateboxCharge, &t.PaymentID, &t.Details, &t.CreatedAt); err != nil {
			return nil, err
		}
		out = append(out, t)
	}
	return out, rows.Err()
}

func (r *Repository) GetSimulationSettings(ctx context.Context, accountID string) (models.SimulationSettings, error) {
	var s models.SimulationSettings
	var autoApprove, autoReject, timeoutEnabled, insufficient, webhook int
	err := r.db.QueryRowContext(ctx, `SELECT auto_approve, auto_reject, processing_delay_ms, random_failure_rate, timeout_enabled, insufficient_balance_enabled, webhook_active, gatebox_environment FROM banco_simulation_settings WHERE account_id = $1`, accountID).
		Scan(&autoApprove, &autoReject, &s.ProcessingDelayMs, &s.RandomFailureRate, &timeoutEnabled, &insufficient, &webhook, &s.GateboxEnvironment)
	if err != nil {
		if err == sql.ErrNoRows {
			return models.SimulationSettings{WebhookActive: true, GateboxEnvironment: "sandbox"}, nil
		}
		return s, err
	}
	s.AutoApprove = autoApprove == 1
	s.AutoReject = autoReject == 1
	s.TimeoutEnabled = timeoutEnabled == 1
	s.InsufficientBalanceEnable = insufficient == 1
	s.WebhookActive = webhook == 1
	return s, nil
}

func (r *Repository) UpdateSimulationSettings(ctx context.Context, accountID string, s models.SimulationSettings) error {
	now := time.Now().UTC().Format(time.RFC3339)
	toInt := func(v bool) int {
		if v {
			return 1
		}
		return 0
	}
	_, err := r.db.ExecContext(ctx, `
INSERT INTO banco_simulation_settings (account_id, auto_approve, auto_reject, processing_delay_ms, random_failure_rate, timeout_enabled, insufficient_balance_enabled, webhook_active, gatebox_environment, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
ON CONFLICT(account_id) DO UPDATE SET
auto_approve = excluded.auto_approve,
auto_reject = excluded.auto_reject,
processing_delay_ms = excluded.processing_delay_ms,
random_failure_rate = excluded.random_failure_rate,
timeout_enabled = excluded.timeout_enabled,
insufficient_balance_enabled = excluded.insufficient_balance_enabled,
webhook_active = excluded.webhook_active,
gatebox_environment = excluded.gatebox_environment,
updated_at = excluded.updated_at`,
		accountID, toInt(s.AutoApprove), toInt(s.AutoReject), s.ProcessingDelayMs, s.RandomFailureRate, toInt(s.TimeoutEnabled), toInt(s.InsufficientBalanceEnable), toInt(s.WebhookActive), s.GateboxEnvironment, now)
	return err
}

func (r *Repository) AppendAuditLog(ctx context.Context, accountID, action, actor, metadata string) error {
	now := time.Now().UTC().Format(time.RFC3339)
	_, err := r.db.ExecContext(ctx, `INSERT INTO banco_audit_logs (id, account_id, action, actor, metadata, created_at) VALUES (md5(random()::text || clock_timestamp()::text), $1, $2, $3, $4, $5)`, accountID, action, actor, metadata, now)
	return err
}

func (r *Repository) SaveGateboxLog(ctx context.Context, paymentID, operation, reqBody, respBody string, statusCode, durationMS int) error {
	now := time.Now().UTC().Format(time.RFC3339)
	_, err := r.db.ExecContext(ctx, `INSERT INTO banco_gatebox_integration_logs (id, payment_id, operation, request_body, response_body, status_code, duration_ms, created_at) VALUES (md5(random()::text || clock_timestamp()::text), $1, $2, $3, $4, $5, $6, $7)`,
		paymentID, operation, reqBody, respBody, statusCode, durationMS, now)
	return err
}

