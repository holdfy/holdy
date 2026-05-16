package model

import (
  "time"
)
type Transaction_status struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsFinal bool `json:"is_final" db:"is_final"`
    IsSuccess bool `json:"is_success" db:"is_success"`
    AllowsRefund bool `json:"allows_refund" db:"allows_refund"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"transaction_status_category_full_count"`
}