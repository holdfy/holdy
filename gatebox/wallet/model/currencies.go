package model

import (
  "time"
)
type Currencies struct {
    ID int64 `json:"id" db:"id"`
    CurrencyCode string `json:"currency_code" db:"currency_code"`
    IsoCode string `json:"iso_code" db:"iso_code"`
    Name string `json:"name" db:"name"`
    Symbol string `json:"symbol" db:"symbol"`
    DecimalPlaces int64 `json:"decimal_places" db:"decimal_places"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"currencies_category_full_count"`
}