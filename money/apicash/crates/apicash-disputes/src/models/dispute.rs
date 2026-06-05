//! Agregado de disputa.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::evidence::Evidence;
use super::resolution::ResolutionType;

/// Quem abriu a disputa no contexto do pedido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeParty {
    Buyer,
    Seller,
}

/// Ciclo de vida da disputa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Closed,
}

/// Motivo estruturado da disputa — enviado pelo comprador ou vendedor ao abrir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeReason {
    ProductNotReceived,
    ProductDamaged,
    WrongProduct,
    NotAsDescribed,
    EmptyBox,
    Other,
}

impl DisputeReason {
    pub fn from_menu_choice(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::ProductNotReceived),
            2 => Some(Self::ProductDamaged),
            3 => Some(Self::WrongProduct),
            4 => Some(Self::EmptyBox),
            5 => Some(Self::Other),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::ProductNotReceived => "product_not_received",
            Self::ProductDamaged     => "product_damaged",
            Self::WrongProduct       => "wrong_product",
            Self::NotAsDescribed     => "not_as_described",
            Self::EmptyBox           => "empty_box",
            Self::Other              => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "product_not_received" => Self::ProductNotReceived,
            "product_damaged"      => Self::ProductDamaged,
            "wrong_product"        => Self::WrongProduct,
            "not_as_described"     => Self::NotAsDescribed,
            "empty_box"            => Self::EmptyBox,
            _                      => Self::Other,
        }
    }
}

/// Veredito da análise de imagens pela IA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiVerdict {
    FavorBuyer,
    FavorSeller,
    Inconclusive,
}

impl AiVerdict {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::FavorBuyer   => "favor_buyer",
            Self::FavorSeller  => "favor_seller",
            Self::Inconclusive => "inconclusive",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "favor_buyer"   => Some(Self::FavorBuyer),
            "favor_seller"  => Some(Self::FavorSeller),
            "inconclusive"  => Some(Self::Inconclusive),
            _               => None,
        }
    }
}

/// Resultado retornado pelo OpenAI Vision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAnalysisResult {
    pub verdict:    AiVerdict,
    pub confidence: f32,
    pub reasoning:  String,
    pub red_flags:  Vec<String>,
}

/// Disputa de escrow: fundos permanecem travados até resolução.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: Uuid,
    pub order_id: Uuid,
    /// Papel de quem abriu (comprador ou vendedor).
    pub opened_by: DisputeParty,
    /// Identificador do usuário que abriu (para eventos e auditoria).
    pub opened_by_user_id: Uuid,
    pub reason: String,
    pub status: DisputeStatus,
    pub evidence: Vec<Evidence>,
    pub opened_at: DateTime<Utc>,
    /// Prazo máximo para submissão de evidências pelo comprador (48h após abertura).
    pub deadline_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    /// Preenchido ao resolver; `Manual` pode não implicar movimentação de escrow.
    pub resolution_type: Option<ResolutionType>,
    pub resolution_notes: Option<String>,
    /// Veredito da IA (preenchido após análise de imagens).
    pub ai_verdict: Option<AiVerdict>,
    pub ai_confidence: Option<f32>,
    /// Raciocínio da IA — visível apenas para admin.
    pub ai_reasoning: Option<String>,
    /// Comprador com score < 200 ao abrir — admin review obrigatório.
    pub high_risk_buyer: bool,
}
