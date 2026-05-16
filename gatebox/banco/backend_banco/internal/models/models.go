package models

type PaymentState string

const (
	PaymentCreated    PaymentState = "CREATED"
	PaymentValidating PaymentState = "VALIDATING"
	PaymentValidated  PaymentState = "VALIDATED"
	PaymentProcessing PaymentState = "PROCESSING"
	PaymentPending    PaymentState = "PENDING"
	PaymentApproved   PaymentState = "APPROVED"
	PaymentRejected   PaymentState = "REJECTED"
	PaymentFailed     PaymentState = "FAILED"
	PaymentTimeout    PaymentState = "TIMEOUT"
	PaymentRefunded   PaymentState = "REFUNDED"
	PaymentCanceled   PaymentState = "CANCELED"
)

type Account struct {
	ID            string `json:"id"`
	UserID        string `json:"user_id"`
	FullName      string `json:"full_name"`
	PersonType    string `json:"person_type"`
	Document      string `json:"document"`
	Email         string `json:"email"`
	Agency        string `json:"agency"`
	AccountNumber string `json:"account_number"`
	PixKey        string `json:"pix_key"`
	Status        string `json:"status"`
}

type Balance struct {
	AccountID      string `json:"account_id"`
	AvailableCents int64  `json:"available_cents"`
	BlockedCents   int64  `json:"blocked_cents"`
}

type Transaction struct {
	ID             string `json:"id"`
	Type           string `json:"type"`
	AmountCents    int64  `json:"amount_cents"`
	Status         string `json:"status"`
	Payer          string `json:"payer,omitempty"`
	Receiver       string `json:"receiver,omitempty"`
	GateboxCharge  string `json:"gatebox_charge_id,omitempty"`
	PaymentID      string `json:"payment_id,omitempty"`
	Details        string `json:"details,omitempty"`
	CreatedAt      string `json:"created_at"`
}

type SimulationSettings struct {
	AutoApprove               bool    `json:"auto_approve"`
	AutoReject                bool    `json:"auto_reject"`
	ProcessingDelayMs         int     `json:"processing_delay_ms"`
	RandomFailureRate         float64 `json:"random_failure_rate"`
	TimeoutEnabled            bool    `json:"timeout_enabled"`
	InsufficientBalanceEnable bool    `json:"insufficient_balance_enabled"`
	WebhookActive             bool    `json:"webhook_active"`
	GateboxEnvironment        string  `json:"gatebox_environment"`
}

