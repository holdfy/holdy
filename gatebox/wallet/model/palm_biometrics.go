package model

import (
  "github.com/jackc/pgx/v5/pgtype"
  "time"
)
type Palm_biometrics struct {
    ID int64 `json:"id" db:"id"`
    BiometricCode string `json:"biometric_code" db:"biometric_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    PalmHash string `json:"palm_hash" db:"palm_hash"`
    AccuracyScore pgtype.Numeric `json:"accuracy_score" db:"accuracy_score"`
    IdHandType int64 `json:"id_hand_type" db:"id_hand_type"`
    EnrollmentDeviceId string `json:"enrollment_device_id" db:"enrollment_device_id"`
    BitmapSignature string `json:"bitmap_signature" db:"bitmap_signature"`
    IsPrimary bool `json:"is_primary" db:"is_primary"`
    IsActive bool `json:"is_active" db:"is_active"`
    RegisteredAt time.Time `json:"registered_at" db:"registered_at"`
    LastUsed time.Time `json:"last_used" db:"last_used"`
    UsageCount int64 `json:"usage_count" db:"usage_count"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"palm_biometrics_category_full_count"`
}