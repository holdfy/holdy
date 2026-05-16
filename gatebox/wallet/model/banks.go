package model

import (
  "time"
)
type Banks struct {
    ID int64 `json:"id" db:"id"`
    BankCodeInternal string `json:"bank_code_internal" db:"bank_code_internal"`
    BankCode string `json:"bank_code" db:"bank_code"`
    Name string `json:"name" db:"name"`
    FullName string `json:"full_name" db:"full_name"`
    Website string `json:"website" db:"website"`
    IsOpenFinance bool `json:"is_open_finance" db:"is_open_finance"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"banks_category_full_count"`
}