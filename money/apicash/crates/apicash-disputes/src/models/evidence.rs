//! Evidências anexadas a uma disputa (fotos, rastreio, mensagens).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tipo de evidência enviada pelo comprador ou vendedor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Photo,
    Video,
    TrackingCode,
    Message,
    Other,
}

impl EvidenceKind {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Photo        => "photo",
            Self::Video        => "video",
            Self::TrackingCode => "tracking_code",
            Self::Message      => "message",
            Self::Other        => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "photo"         => Self::Photo,
            "video"         => Self::Video,
            "tracking_code" => Self::TrackingCode,
            "message"       => Self::Message,
            _               => Self::Other,
        }
    }
}

/// Lado que enviou a evidência.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceParty {
    Buyer,
    Seller,
}

impl EvidenceParty {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Buyer  => "buyer",
            Self::Seller => "seller",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "seller" => Self::Seller,
            _        => Self::Buyer,
        }
    }
}

/// Evidência persistida no banco (tabela `dispute_evidence`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRow {
    pub id:          Uuid,
    pub dispute_id:  Uuid,
    pub uploaded_by: Uuid,
    pub party:       EvidenceParty,
    pub kind:        EvidenceKind,
    /// Chave no MinIO (ex: `disputes/<dispute_id>/photo_<uuid>.jpg`). None para textos.
    pub minio_key:   Option<String>,
    /// URL pública no MinIO ou None para tipos sem arquivo.
    pub minio_url:   Option<String>,
    /// Texto livre (código de rastreio, mensagem) ou None para fotos/vídeos.
    pub content:     Option<String>,
    /// SHA-256 hex do conteúdo binário (para fotos/vídeos) ou do texto UTF-8.
    pub sha256:      String,
    /// Marcado como suspeito pela IA (imagem genérica, stock, inconsistente).
    pub ai_flagged:  bool,
    pub created_at:  DateTime<Utc>,
}

/// Evidência leve (JSONB legado na coluna `evidence` da tabela `disputes`).
/// Mantido para compatibilidade com o repositório existente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub kind: EvidenceKind,
    pub description: String,
    /// URL, hash ou código externo (ex.: rastreio).
    pub reference: Option<String>,
}
