package model

import (
  "time"
)
type System_configurations struct {
    ID int64 `json:"id" db:"id"`
    ConfigCode string `json:"config_code" db:"config_code"`
    ApplicationId int64 `json:"application_id" db:"application_id"`
    ConfigKey string `json:"config_key" db:"config_key"`
    ConfigValue string `json:"config_value" db:"config_value"`
    ConfigType string `json:"config_type" db:"config_type"`
    Description string `json:"description" db:"description"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"system_configurations_category_full_count"`
}