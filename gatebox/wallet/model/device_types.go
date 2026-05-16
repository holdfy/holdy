package model

import (
  "time"
)
type Device_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsMobile bool `json:"is_mobile" db:"is_mobile"`
    SecurityLevel int64 `json:"security_level" db:"security_level"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"device_types_category_full_count"`
}