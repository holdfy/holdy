package model

import (
  "time"
)
type Notification_templates struct {
    ID int64 `json:"id" db:"id"`
    TemplateCode string `json:"template_code" db:"template_code"`
    ApplicationId int64 `json:"application_id" db:"application_id"`
    TemplateKey string `json:"template_key" db:"template_key"`
    IdChannel int64 `json:"id_channel" db:"id_channel"`
    Subject string `json:"subject" db:"subject"`
    TemplateBody string `json:"template_body" db:"template_body"`
    TemplateVariables string `json:"template_variables" db:"template_variables"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"notification_templates_category_full_count"`
}