//! Proposal DTOs — two-party escrow negotiation before order creation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Seller creates a proposal for a specific buyer + amount.
/// `buyer_id` is optional — if omitted, a nil UUID is stored and any authenticated
/// buyer can accept the proposal (open/link-based proposal flow).
#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    /// UUID of the buyer (counterparty). Omit for open/link-based proposals.
    #[serde(default)]
    pub buyer_id: Option<Uuid>,
    /// Decimal amount string (e.g. "100.50").
    pub amount: String,
    /// Optional item/service description shown to buyer.
    #[serde(default)]
    pub description: Option<String>,
    /// Seller PIX key — saved in wa_contacts for automatic off-ramp after delivery confirmation.
    #[serde(default)]
    pub seller_pix_key: Option<String>,
    /// Listing UUID from importer — when set, the listing is linked to the order after acceptance.
    #[serde(default)]
    pub listing_id: Option<Uuid>,
    /// Número de WhatsApp do vendedor (ex.: `+5541999990000`) — salvo em wa_contacts para notificações de rastreio.
    #[serde(default)]
    pub seller_phone: Option<String>,
}

impl CreateProposalRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.amount.trim().is_empty() {
            return Err("amount is required");
        }
        Ok(())
    }
}

/// Buyer accepts a proposal — creates the escrow order and returns PIX QR.
#[derive(Debug, Deserialize, Default)]
pub struct AcceptProposalRequest {
    /// Buyer CPF (11 digits) for anti-fraud; defaults to placeholder if omitted.
    #[serde(default)]
    pub cpf: Option<String>,
    /// Social links for anti-fraud scoring (optional).
    #[serde(default)]
    pub social_links: Option<Vec<String>>,
    /// Número de WhatsApp do comprador (ex.: `+5541999990000`) — salvo em wa_contacts para notificações de rastreio.
    #[serde(default)]
    pub buyer_phone: Option<String>,
    /// Canal que originou o pedido: `whatsapp` | `site` | `app_ios` | `app_android`.
    /// Omitido/desconhecido → `site` (fallback histórico).
    #[serde(default)]
    pub platform: Option<String>,
}

impl AcceptProposalRequest {
    pub fn platform_origin(&self) -> apicash_shared::PlatformOrigin {
        self.platform
            .as_deref()
            .and_then(|p| p.parse().ok())
            .unwrap_or(apicash_shared::PlatformOrigin::Site)
    }
}

/// Proposal status lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

impl std::fmt::Display for ProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProposalStatus::Pending => write!(f, "pending"),
            ProposalStatus::Accepted => write!(f, "accepted"),
            ProposalStatus::Rejected => write!(f, "rejected"),
            ProposalStatus::Expired => write!(f, "expired"),
        }
    }
}

/// In-memory proposal record.
#[derive(Debug, Clone)]
pub struct StoredProposal {
    pub id: Uuid,
    pub seller_id: Uuid,
    /// CPF/CNPJ of the seller — captured from JWT at creation time.
    pub seller_document: Option<String>,
    pub buyer_id: Uuid,
    /// Canonical decimal string (e.g. "100.50").
    pub amount: String,
    pub description: Option<String>,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    /// Expires 1 hour after creation.
    pub expires_at: DateTime<Utc>,
    /// Filled in when buyer accepts and an order is created.
    pub order_id: Option<Uuid>,
    /// Listing UUID from importer — linked to the order after acceptance.
    pub listing_id: Option<Uuid>,
}

impl StoredProposal {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Response for GET /proposals/{id} and POST /proposals.
#[derive(Debug, Serialize)]
pub struct ProposalResponse {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub buyer_id: Uuid,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: ProposalStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<Uuid>,
    /// CPF/CNPJ of the seller — shown to buyer for identity verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_document: Option<String>,
    /// First photo URL from the linked listing, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_photo: Option<String>,
    /// Seller's WhatsApp — shown to buyer for contact/confirmation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seller_phone: Option<String>,
}

impl From<&StoredProposal> for ProposalResponse {
    fn from(p: &StoredProposal) -> Self {
        Self {
            id: p.id,
            seller_id: p.seller_id,
            buyer_id: p.buyer_id,
            amount: p.amount.clone(),
            description: p.description.clone(),
            status: p.status.clone(),
            created_at: p.created_at,
            expires_at: p.expires_at,
            order_id: p.order_id,
            seller_document: p.seller_document.clone(),
            listing_photo: None,
            seller_phone: None,
        }
    }
}

/// Response returned when buyer accepts — includes the full order + PIX QR.
#[derive(Debug, Serialize)]
pub struct AcceptProposalResponse {
    pub proposal_id: Uuid,
    pub order_id: Uuid,
    pub pix_br_code: String,
    pub amount: String,
    pub status: ProposalStatus,
    pub funding_instruction: String,
}
