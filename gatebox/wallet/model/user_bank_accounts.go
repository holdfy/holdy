package model

import (
  "time"
)
type User_bank_accounts struct {
    ID int64 `json:"id" db:"id"`
    BankAccountCode string `json:"bank_account_code" db:"bank_account_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    WalletId int64 `json:"wallet_id" db:"wallet_id"`
    IdBank int64 `json:"id_bank" db:"id_bank"`
    Agency string `json:"agency" db:"agency"`
    AccountNumber string `json:"account_number" db:"account_number"`
    IdAccountType int64 `json:"id_account_type" db:"id_account_type"`
    HolderName string `json:"holder_name" db:"holder_name"`
    HolderDocument string `json:"holder_document" db:"holder_document"`
    ConsentId string `json:"consent_id" db:"consent_id"`
    ConsentExpiresAt time.Time `json:"consent_expires_at" db:"consent_expires_at"`
    IsVerified bool `json:"is_verified" db:"is_verified"`
    IsActive bool `json:"is_active" db:"is_active"`
    LastUsed time.Time `json:"last_used" db:"last_used"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"user_bank_accounts_category_full_count"`
}