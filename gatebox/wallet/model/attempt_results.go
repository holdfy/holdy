package model

import (
  "time"
)
type Attempt_results struct {
    ID int64 `json:"id" db:"id"`
    ResultCode string `json:"result_code" db:"result_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    IsSuccess bool `json:"is_success" db:"is_success"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"attempt_results_category_full_count"`
}