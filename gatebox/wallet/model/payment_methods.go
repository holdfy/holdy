package model

import (
  "time"
)
type Payment_methods struct {
    ID int64 `json:"id" db:"id"`
    MethodCode string `json:"method_code" db:"method_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    RequiresExternalAuth bool `json:"requires_external_auth" db:"requires_external_auth"`
    ProcessingTimeMinutes int64 `json:"processing_time_minutes" db:"processing_time_minutes"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"payment_methods_category_full_count"`
}