package model

import (
  "github.com/jackc/pgx/v5/pgtype"
  "time"
)
type Wallets struct {
    ID int64 `json:"id" db:"id"`
    WalletCode string `json:"wallet_code" db:"wallet_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    ApplicationId int64 `json:"application_id" db:"application_id"`
    Name string `json:"name" db:"name"`
    IdWalletType int64 `json:"id_wallet_type" db:"id_wallet_type"`
    IdCurrency int64 `json:"id_currency" db:"id_currency"`
    IdStatus int64 `json:"id_status" db:"id_status"`
    Balance pgtype.Numeric `json:"balance" db:"balance"`
    AvailableBalance pgtype.Numeric `json:"available_balance" db:"available_balance"`
    DailyLimit pgtype.Numeric `json:"daily_limit" db:"daily_limit"`
    MonthlyLimit pgtype.Numeric `json:"monthly_limit" db:"monthly_limit"`
    SignatureRequiredAbove pgtype.Numeric `json:"signature_required_above" db:"signature_required_above"`
    IsPrimary bool `json:"is_primary" db:"is_primary"`
    Metadata string `json:"metadata" db:"metadata"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"wallets_category_full_count"`
}