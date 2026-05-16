package model

import (
  "time"
)
type Applications struct {
    ID int64 `json:"id" db:"id"`
    AppCode string `json:"app_code" db:"app_code"`
    Name string `json:"name" db:"name"`
    Code string `json:"code" db:"code"`
    Description string `json:"description" db:"description"`
    ApiKey string `json:"api_key" db:"api_key"`
    IsActive bool `json:"is_active" db:"is_active"`
    Settings string `json:"settings" db:"settings"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"applications_category_full_count"`
}