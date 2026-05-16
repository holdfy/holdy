package model

import (
  "time"
)
type User_documents struct {
    ID int64 `json:"id" db:"id"`
    DocumentCode string `json:"document_code" db:"document_code"`
    UserId int64 `json:"user_id" db:"user_id"`
    IdDocumentType int64 `json:"id_document_type" db:"id_document_type"`
    DocumentNumber string `json:"document_number" db:"document_number"`
    FilePath string `json:"file_path" db:"file_path"`
    FileHash string `json:"file_hash" db:"file_hash"`
    IdStatusDocuments int64 `json:"id_status_documents" db:"id_status_documents"`
    VerifiedAt time.Time `json:"verified_at" db:"verified_at"`
    VerifiedBy int64 `json:"verified_by" db:"verified_by"`
    RejectionReason string `json:"rejection_reason" db:"rejection_reason"`
    Metadata string `json:"metadata" db:"metadata"`
    CreatedAt time.Time `json:"created_at" db:"created_at"`
    UpdatedAt time.Time `json:"updated_at" db:"updated_at"`
    FullCount   int64  `json:"-" db:"user_documents_category_full_count"`
}