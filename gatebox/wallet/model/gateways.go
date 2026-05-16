package model

import (
  "time"
)
type Gateways struct {
    ID int64 `json:"id" db:"id"`
    GatewayCode string `json:"gateway_code" db:"gateway_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    ApiEndpoint string `json:"api_endpoint" db:"api_endpoint"`
    TimeoutSeconds int64 `json:"timeout_seconds" db:"timeout_seconds"`
    MaxRetries int64 `json:"max_retries" db:"max_retries"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"gateways_category_full_count"`
}