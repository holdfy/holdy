package model

import (
	"time"

	"github.com/jackc/pgx/v5/pgtype"
)

type Transactions struct {
	ID                    int64          `json:"id" db:"id"`
	TransactionCode       string         `json:"transaction_code" db:"transaction_code"`
	WalletId              int64          `json:"wallet_id" db:"wallet_id"`
	ExternalTransactionId string         `json:"external_transaction_id" db:"external_transaction_id"`
	IdTransactionType     int64          `json:"id_transaction_type" db:"id_transaction_type"`
	IdPaymentMethod       int64          `json:"id_payment_method" db:"id_payment_method"`
	IdStatus              int64          `json:"id_status" db:"id_status"`
	Amount                pgtype.Numeric `json:"amount" db:"amount"`
	IdCurrency            int64          `json:"id_currency" db:"id_currency"`
	FeeAmount             pgtype.Numeric `json:"fee_amount" db:"fee_amount"`
	NetAmount             pgtype.Numeric `json:"net_amount" db:"net_amount"`
	PayerUserId           int64          `json:"payer_user_id" db:"payer_user_id"`
	PayerWalletId         int64          `json:"payer_wallet_id" db:"payer_wallet_id"`
	PayeeUserId           int64          `json:"payee_user_id" db:"payee_user_id"`
	PayeeWalletId         int64          `json:"payee_wallet_id" db:"payee_wallet_id"`
	PayeeExternalAccount  string         `json:"payee_external_account" db:"payee_external_account"`
	PaymentMethodId       int64          `json:"payment_method_id" db:"payment_method_id"`
	MerchantId            string         `json:"merchant_id" db:"merchant_id"`
	MerchantName          string         `json:"merchant_name" db:"merchant_name"`
	MerchantCategory      string         `json:"merchant_category" db:"merchant_category"`
	DeviceId              string         `json:"device_id" db:"device_id"`
	DeviceInfo            string         `json:"device_info" db:"device_info"`
	LocationData          string         `json:"location_data" db:"location_data"`
	IpAddress             string         `json:"ip_address" db:"ip_address"`
	RequiresSignature     bool           `json:"requires_signature" db:"requires_signature"`
	SignatureProvided     bool           `json:"signature_provided" db:"signature_provided"`
	IdSignatureMethod     int64          `json:"id_signature_method" db:"id_signature_method"`
	AuthorizedAt          time.Time      `json:"authorized_at" db:"authorized_at"`
	CompletedAt           time.Time      `json:"completed_at" db:"completed_at"`
	CancelledAt           time.Time      `json:"cancelled_at" db:"cancelled_at"`
	Description           string         `json:"description" db:"description"`
	Metadata              string         `json:"metadata" db:"metadata"`
	CreatedAt             time.Time      `json:"created_at" db:"created_at"`
	UpdatedAt             time.Time      `json:"updated_at" db:"updated_at"`
	FullCount             int64          `json:"-" db:"transactions_category_full_count"`
}
