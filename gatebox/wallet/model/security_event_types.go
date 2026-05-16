package model

import (
  "time"
)
type Security_event_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    DefaultSeverity int64 `json:"default_severity" db:"default_severity"`
    AutoBlock bool `json:"auto_block" db:"auto_block"`
    RequiresInvestigation bool `json:"requires_investigation" db:"requires_investigation"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"security_event_types_category_full_count"`
}