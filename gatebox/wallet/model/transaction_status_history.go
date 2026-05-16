package model

import (
  "time"
)
type Transaction_status_history struct {
    ID int64 `json:"id" db:"id"`
    StatusHistoryCode string `json:"status_history_code" db:"status_history_code"`
    TransactionId int64 `json:"transaction_id" db:"transaction_id"`
    IdPreviousStatus int64 `json:"id_previous_status" db:"id_previous_status"`
    IdNewStatus int64 `json:"id_new_status" db:"id_new_status"`
    Reason string `json:"reason" db:"reason"`
    IdChangedBy int64 `json:"id_changed_by" db:"id_changed_by"`
    GatewayResponse string `json:"gateway_response" db:"gateway_response"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"transaction_status_history_category_full_count"`
}