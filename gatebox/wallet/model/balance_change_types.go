package model

import (
  "time"
)
type Balance_change_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsPositive bool `json:"is_positive" db:"is_positive"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"balance_change_types_category_full_count"`
}