package model

import (
  "time"
)
type User_restrictions struct {
    ID int64 `json:"id" db:"id"`
    RestrictionCode string `json:"restriction_code" db:"restriction_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    IdRestrictionType int64 `json:"id_restriction_type" db:"id_restriction_type"`
    RestrictionReason string `json:"restriction_reason" db:"restriction_reason"`
    Restrictions string `json:"restrictions" db:"restrictions"`
    IsActive bool `json:"is_active" db:"is_active"`
    ExpiresAt time.Time `json:"expires_at" db:"expires_at"`
    CreatedBy int64 `json:"created_by" db:"created_by"`
    RemovedBy int64 `json:"removed_by" db:"removed_by"`
    RemovedAt time.Time `json:"removed_at" db:"removed_at"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    FullCount   int64  `json:"-" db:"user_restrictions_category_full_count"`
}