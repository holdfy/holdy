package model

import (
	"time"
)

type User_sessions struct {
	ID                int64     `json:"id" db:"id"`
	SessionCode       string    `json:"session_code" db:"session_code"`
	UserId            int64     `json:"user_id" db:"user_id"`
	ApplicationId     int64     `json:"application_id" db:"application_id"`
	SessionToken      string    `json:"session_token" db:"session_token"`
	DeviceFingerprint string    `json:"device_fingerprint" db:"device_fingerprint"`
	IdDeviceType      int64     `json:"id_device_type" db:"id_device_type"`
	DeviceInfo        string    `json:"device_info" db:"device_info"`
	IpAddress         string    `json:"ip_address" db:"ip_address"`
	LocationData      string    `json:"location_data" db:"location_data"`
	IdStatus          int64     `json:"id_status" db:"id_status"`
	ExpiresAt         time.Time `json:"expires_at" db:"expires_at"`
	LastActivity      time.Time `json:"last_activity" db:"last_activity"`
	CreatedAt         time.Time `json:"created_at" db:"created_at"`
	FullCount         int64     `json:"-" db:"user_sessions_category_full_count"`
}
