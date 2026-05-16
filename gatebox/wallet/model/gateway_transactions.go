package model

import (
  "time"
)
type Gateway_transactions struct {
    ID int64 `json:"id" db:"id"`
    GatewayTransactionCode string `json:"gateway_transaction_code" db:"gateway_transaction_code"`
    TransactionId int64 `json:"transaction_id" db:"transaction_id"`
    IdGateway int64 `json:"id_gateway" db:"id_gateway"`
    GatewayTransactionId string `json:"gateway_transaction_id" db:"gateway_transaction_id"`
    IdGatewayStatus int64 `json:"id_gateway_status" db:"id_gateway_status"`
    GatewayResponse string `json:"gateway_response" db:"gateway_response"`
    GatewayRequest string `json:"gateway_request" db:"gateway_request"`
    ProcessingTimeMs int64 `json:"processing_time_ms" db:"processing_time_ms"`
    RetryCount int64 `json:"retry_count" db:"retry_count"`
    LastRetryAt time.Time `json:"last_retry_at" db:"last_retry_at"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"gateway_transactions_category_full_count"`
}