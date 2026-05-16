package model

import (
  "time"
)
type Document_status struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsFinal bool `json:"is_final" db:"is_final"`
    RequiresAction bool `json:"requires_action" db:"requires_action"`
    NextPossibleStatus string `json:"next_possible_status" db:"next_possible_status"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"document_status_category_full_count"`
}