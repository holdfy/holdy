package model

import (
  "time"
)
type Document_types struct {
    ID int64 `json:"id" db:"id"`
    TypeCode string `json:"type_code" db:"type_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsRequired bool `json:"is_required" db:"is_required"`
    MaxFileSizeMb int64 `json:"max_file_size_mb" db:"max_file_size_mb"`
    AllowedExtensions string `json:"allowed_extensions" db:"allowed_extensions"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"document_types_category_full_count"`
}