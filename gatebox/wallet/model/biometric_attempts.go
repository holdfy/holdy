package model

import (
	"time"

	"github.com/jackc/pgx/v5/pgtype"
)

type Biometric_attempts struct {
	ID              int64          `json:"id" db:"id"`
	AttemptCode     string         `json:"attempt_code" db:"attempt_code"`
	UserId          int64          `json:"user_id" db:"user_id"`
	PalmHash        string         `json:"palm_hash" db:"palm_hash"`
	AccuracyScore   pgtype.Numeric `json:"accuracy_score" db:"accuracy_score"`
	DeviceId        string         `json:"device_id" db:"device_id"`
	IdAttemptResult int64          `json:"id_attempt_result" db:"id_attempt_result"`
	IdFailureReason int64          `json:"id_failure_reason" db:"id_failure_reason"`
	IpAddress       string         `json:"ip_address" db:"ip_address"`
	UserAgent       string         `json:"user_agent" db:"user_agent"`
	LocationData    string         `json:"location_data" db:"location_data"`
	CreatedAt       time.Time      `json:"created_at" db:"created_at"`
	FullCount       int64          `json:"-" db:"biometric_attempts_category_full_count"`
}
