package model

import (
  "time"
)
type Notification_history struct {
    ID int64 `json:"id" db:"id"`
    NotificationCode string `json:"notification_code" db:"notification_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    TransactionId int64 `json:"transaction_id" db:"transaction_id"`
    TemplateId int64 `json:"template_id" db:"template_id"`
    IdChannel int64 `json:"id_channel" db:"id_channel"`
    Recipient string `json:"recipient" db:"recipient"`
    Subject string `json:"subject" db:"subject"`
    MessageBody string `json:"message_body" db:"message_body"`
    IdStatus int64 `json:"id_status" db:"id_status"`
    ProviderResponse string `json:"provider_response" db:"provider_response"`
    SentAt time.Time `json:"sent_at" db:"sent_at"`
    DeliveredAt time.Time `json:"delivered_at" db:"delivered_at"`
    FailedAt time.Time `json:"failed_at" db:"failed_at"`
    FailureReason string `json:"failure_reason" db:"failure_reason"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"notification_history_category_full_count"`
}