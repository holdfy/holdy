package model

import (
	"time"
)

type Security_events struct {
	ID                int64     `json:"id" db:"id"`
	SecurityEventCode string    `json:"security_event_code" db:"security_event_code"`
	UserId            int64     `json:"user_id" db:"user_id"`
	IdEventType       int64     `json:"id_event_type" db:"id_event_type"`
	IdSeverity        int64     `json:"id_severity" db:"id_severity"`
	Description       string    `json:"description" db:"description"`
	SourceIp          string    `json:"source_ip" db:"source_ip"`
	DeviceInfo        string    `json:"device_info" db:"device_info"`
	Metadata          string    `json:"metadata" db:"metadata"`
	IsResolved        bool      `json:"is_resolved" db:"is_resolved"`
	ResolvedAt        time.Time `json:"resolved_at" db:"resolved_at"`
	ResolvedBy        int64     `json:"resolved_by" db:"resolved_by"`
	CreatedAt         time.Time `json:"created_at" db:"created_at"`
	FullCount         int64     `json:"-" db:"security_events_category_full_count"`
}
