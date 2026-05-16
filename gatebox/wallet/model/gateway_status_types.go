package model

import (
  "time"
)
type Gateway_status_types struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsSuccess bool `json:"is_success" db:"is_success"`
    IsFinal bool `json:"is_final" db:"is_final"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"gateway_status_types_category_full_count"`
}