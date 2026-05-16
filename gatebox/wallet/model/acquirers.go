package model

import (
  "time"
)
type Acquirers struct {
    ID int64 `json:"id" db:"id"`
    AcquirerCode string `json:"acquirer_code" db:"acquirer_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    ApiEndpoint string `json:"api_endpoint" db:"api_endpoint"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"acquirers_category_full_count"`
}