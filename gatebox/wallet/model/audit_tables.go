package model

import (
  "time"
)
type Audit_tables struct {
    ID int64 `json:"id" db:"id"`
    TableCode string `json:"table_code" db:"table_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    SensitivityLevel int64 `json:"sensitivity_level" db:"sensitivity_level"`
    RetentionDays int64 `json:"retention_days" db:"retention_days"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"audit_tables_category_full_count"`
}