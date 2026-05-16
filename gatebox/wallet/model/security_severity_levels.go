package model

import (
  "time"
)
type Security_severity_levels struct {
    ID int64 `json:"id" db:"id"`
    SeverityCode string `json:"severity_code" db:"severity_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    LevelNumber int64 `json:"level_number" db:"level_number"`
    NotificationRequired bool `json:"notification_required" db:"notification_required"`
    EscalationRequired bool `json:"escalation_required" db:"escalation_required"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"security_severity_levels_category_full_count"`
}