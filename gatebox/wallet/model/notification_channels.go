package model

import (
  "time"
)
type Notification_channels struct {
    ID int64 `json:"id" db:"id"`
    ChannelCode string `json:"channel_code" db:"channel_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    RequiresSubject bool `json:"requires_subject" db:"requires_subject"`
    MaxBodyLength int64 `json:"max_body_length" db:"max_body_length"`
    DeliveryTimeSeconds int64 `json:"delivery_time_seconds" db:"delivery_time_seconds"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"notification_channels_category_full_count"`
}