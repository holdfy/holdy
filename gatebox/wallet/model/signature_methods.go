package model

import (
  "time"
)
type Signature_methods struct {
    ID int64 `json:"id" db:"id"`
    MethodCode string `json:"method_code" db:"method_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    SecurityLevel int64 `json:"security_level" db:"security_level"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"signature_methods_category_full_count"`
}