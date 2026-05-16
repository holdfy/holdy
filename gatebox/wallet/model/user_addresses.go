package model

import (
  "time"
)
type User_addresses struct {
    ID int64 `json:"id" db:"id"`
    AddressCode string `json:"address_code" db:"address_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    IdAddressType int64 `json:"id_address_type" db:"id_address_type"`
    Street string `json:"street" db:"street"`
    Number string `json:"number" db:"number"`
    Complement string `json:"complement" db:"complement"`
    Neighborhood string `json:"neighborhood" db:"neighborhood"`
    City string `json:"city" db:"city"`
    State string `json:"state" db:"state"`
    ZipCode string `json:"zip_code" db:"zip_code"`
    Country string `json:"country" db:"country"`
    IsPrimary bool `json:"is_primary" db:"is_primary"`
    IsActive bool `json:"is_active" db:"is_active"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"user_addresses_category_full_count"`
}