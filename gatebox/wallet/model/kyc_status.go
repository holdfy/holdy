package model

import (
  "github.com/jackc/pgx/v5/pgtype"
  "time"
)
type Kyc_status struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    AllowsTransactions bool `json:"allows_transactions" db:"allows_transactions"`
    MaxTransactionAmount pgtype.Numeric `json:"max_transaction_amount" db:"max_transaction_amount"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"kyc_status_category_full_count"`
}