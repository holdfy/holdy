package model

import (
	"time"
)

type Audit_log struct {
	ID            int64     `json:"id" db:"id"`
	AuditCode     string    `json:"audit_code" db:"audit_code"`
	IdTable       int64     `json:"id_table" db:"id_table"`
	RecordId      int64     `json:"record_id" db:"record_id"`
	IdAction      int64     `json:"id_action" db:"id_action"`
	OldValues     string    `json:"old_values" db:"old_values"`
	NewValues     string    `json:"new_values" db:"new_values"`
	ChangedFields string    `json:"changed_fields" db:"changed_fields"`
	UserId        int64     `json:"user_id" db:"user_id"`
	ApplicationId int64     `json:"application_id" db:"application_id"`
	IpAddress     string    `json:"ip_address" db:"ip_address"`
	UserAgent     string    `json:"user_agent" db:"user_agent"`
	SessionId     string    `json:"session_id" db:"session_id"`
	CreatedAt     time.Time `json:"created_at" db:"created_at"`
	FullCount     int64     `json:"-" db:"audit_log_category_full_count"`
}
