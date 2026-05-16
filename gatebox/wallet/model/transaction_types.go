package model

import (
  "time"
)
type Transaction_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    AffectsBalance bool `json:"affects_balance" db:"affects_balance"`
    RequiresRecipient bool `json:"requires_recipient" db:"requires_recipient"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"transaction_types_category_full_count"`
}