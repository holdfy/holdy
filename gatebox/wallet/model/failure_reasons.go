package model

import (
  "time"
)
type Failure_reasons struct {
    ID int64 `json:"id" db:"id"`
    ReasonCode string `json:"reason_code" db:"reason_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsCritical bool `json:"is_critical" db:"is_critical"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"failure_reasons_category_full_count"`
}