package model

import (
  "github.com/jackc/pgx/v5/pgtype"
  "time"
)
type Card_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    DefaultDailyLimit pgtype.Numeric `json:"default_daily_limit" db:"default_daily_limit"`
    DefaultMonthlyLimit pgtype.Numeric `json:"default_monthly_limit" db:"default_monthly_limit"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"card_types_category_full_count"`
}