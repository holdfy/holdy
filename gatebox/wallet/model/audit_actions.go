package model

import (
  "time"
)
type Audit_actions struct {
    ID int64 `json:"id" db:"id"`
    ActionCode string `json:"action_code" db:"action_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    SeverityLevel int64 `json:"severity_level" db:"severity_level"`
    RequiresUser bool `json:"requires_user" db:"requires_user"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"audit_actions_category_full_count"`
}