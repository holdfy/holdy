package model

import (
  "time"
)
type Restriction_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    AffectsTransactions bool `json:"affects_transactions" db:"affects_transactions"`
    AffectsLogin bool `json:"affects_login" db:"affects_login"`
    AffectsBiometric bool `json:"affects_biometric" db:"affects_biometric"`
    CanAutoExpire bool `json:"can_auto_expire" db:"can_auto_expire"`
    DefaultDurationHours int64 `json:"default_duration_hours" db:"default_duration_hours"`
    SeverityLevel int64 `json:"severity_level" db:"severity_level"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"restriction_types_category_full_count"`
}