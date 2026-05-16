package model

import (
  "time"
)
type Wallet_status struct {
    ID int64 `json:"id" db:"id"`
    StatusCode string `json:"status_code" db:"status_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    AllowsTransactions bool `json:"allows_transactions" db:"allows_transactions"`
    AllowsDeposits bool `json:"allows_deposits" db:"allows_deposits"`
    AllowsWithdrawals bool `json:"allows_withdrawals" db:"allows_withdrawals"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"wallet_status_category_full_count"`
}