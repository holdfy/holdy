package model

import (
  "github.com/jackc/pgx/v5/pgtype"
  "time"
)
type Wallet_balance_history struct {
    ID int64 `json:"id" db:"id"`
    BalanceHistoryCode string `json:"balance_history_code" db:"balance_history_code"`
    WalletId int64 `json:"wallet_id" db:"wallet_id"`
    PreviousBalance pgtype.Numeric `json:"previous_balance" db:"previous_balance"`
    NewBalance pgtype.Numeric `json:"new_balance" db:"new_balance"`
    ChangeAmount pgtype.Numeric `json:"change_amount" db:"change_amount"`
    IdChangeType int64 `json:"id_change_type" db:"id_change_type"`
    ReferenceId int64 `json:"reference_id" db:"reference_id"`
    Description string `json:"description" db:"description"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"wallet_balance_history_category_full_count"`
}