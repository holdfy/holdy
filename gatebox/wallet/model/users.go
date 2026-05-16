package model

import (
  "time"
)
type Users struct {
    ID int64 `json:"id" db:"id"`
    UserCode string `json:"user_code" db:"user_code"`
    Cpf string `json:"cpf" db:"cpf"`
    FullName string `json:"full_name" db:"full_name"`
    Email string `json:"email" db:"email"`
    Phone string `json:"phone" db:"phone"`
    BirthDate time.Time `json:"birth_date" db:"birth_date"`
    IdStatus int64 `json:"id_status" db:"id_status"`
    IdKycStatus int64 `json:"id_kyc_status" db:"id_kyc_status"`
    KycLevel int64 `json:"kyc_level" db:"kyc_level"`
    AppPasswordHash string `json:"app_password_hash" db:"app_password_hash"`
    BiometricFailures int64 `json:"biometric_failures" db:"biometric_failures"`
    LastLogin time.Time `json:"last_login" db:"last_login"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"users_category_full_count"`
}