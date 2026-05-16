package model

import (
  "time"
)
type External_wallets struct {
    ID int64 `json:"id" db:"id"`
    ExternalWalletCode string `json:"external_wallet_code" db:"external_wallet_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    IdProvider int64 `json:"id_provider" db:"id_provider"`
    ExternalAccountId string `json:"external_account_id" db:"external_account_id"`
    AccountInfo string `json:"account_info" db:"account_info"`
    AccessTokenEncrypted string `json:"access_token_encrypted" db:"access_token_encrypted"`
    RefreshTokenEncrypted string `json:"refresh_token_encrypted" db:"refresh_token_encrypted"`
    TokenExpiresAt time.Time `json:"token_expires_at" db:"token_expires_at"`
    IsActive bool `json:"is_active" db:"is_active"`
    LastSync time.Time `json:"last_sync" db:"last_sync"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"external_wallets_category_full_count"`
}