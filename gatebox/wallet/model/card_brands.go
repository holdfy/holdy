package model

import (
  "time"
)
type Card_brands struct {
    ID int64 `json:"id" db:"id"`
    BrandCode string `json:"brand_code" db:"brand_code"`
    Name string `json:"name" db:"name"`
    Description string `json:"description" db:"description"`
    LogoUrl string `json:"logo_url" db:"logo_url"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"card_brands_category_full_count"`
}