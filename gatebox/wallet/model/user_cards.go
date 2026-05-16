package model

import (
  "time"
)
type User_cards struct {
    ID int64 `json:"id" db:"id"`
    CardCode string `json:"card_code" db:"card_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    WalletId int64 `json:"wallet_id" db:"wallet_id"`
    CardToken string `json:"card_token" db:"card_token"`
    IdCardBrand int64 `json:"id_card_brand" db:"id_card_brand"`
    MaskedNumber string `json:"masked_number" db:"masked_number"`
    HolderName string `json:"holder_name" db:"holder_name"`
    ExpiryMonth int64 `json:"expiry_month" db:"expiry_month"`
    ExpiryYear int64 `json:"expiry_year" db:"expiry_year"`
    IdCardType int64 `json:"id_card_type" db:"id_card_type"`
    IdAcquirer int64 `json:"id_acquirer" db:"id_acquirer"`
    IsPrimary bool `json:"is_primary" db:"is_primary"`
    IsActive bool `json:"is_active" db:"is_active"`
    LastUsed time.Time `json:"last_used" db:"last_used"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"user_cards_category_full_count"`
}