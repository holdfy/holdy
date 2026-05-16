package model

import (
  "time"
)
type Notification_status struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsFinal bool `json:"is_final" db:"is_final"`
    IsSuccess bool `json:"is_success" db:"is_success"`
    RequiresRetry bool `json:"requires_retry" db:"requires_retry"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"notification_status_category_full_count"`
}