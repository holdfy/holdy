package model

import (
  "time"
)
type User_status struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"user_status_category_full_count"`
}