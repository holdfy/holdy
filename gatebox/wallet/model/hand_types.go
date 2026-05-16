package model

import (
  "time"
)
type Hand_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"hand_types_category_full_count"`
}