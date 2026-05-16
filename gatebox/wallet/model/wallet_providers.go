package model

import (
  "time"
)
type Wallet_providers struct {
    ID int64 `json:"id" db:"id"`
    ProviderCode string `json:"provider_code" db:"provider_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    ApiEndpoint string `json:"api_endpoint" db:"api_endpoint"`
    RequiresToken bool `json:"requires_token" db:"requires_token"`
    TokenDurationHours int64 `json:"token_duration_hours" db:"token_duration_hours"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"wallet_providers_category_full_count"`
}