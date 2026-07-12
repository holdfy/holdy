//! Lifecycle enums shared across services.

use std::fmt;

use serde::{Deserialize, Serialize};

/// High-level state of a trade [`Order`](crate::models::Order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Draft,
    PendingFunding,
    Funded,
    InCustody,
    Completed,
    Cancelled,
    Failed,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::Draft => write!(f, "draft"),
            OrderStatus::PendingFunding => write!(f, "pending_funding"),
            OrderStatus::Funded => write!(f, "funded"),
            OrderStatus::InCustody => write!(f, "in_custody"),
            OrderStatus::Completed => write!(f, "completed"),
            OrderStatus::Cancelled => write!(f, "cancelled"),
            OrderStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Client channel that originated an order (analytics — "de onde vem a venda").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformOrigin {
    Whatsapp,
    Site,
    AppIos,
    AppAndroid,
}

impl fmt::Display for PlatformOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformOrigin::Whatsapp => write!(f, "whatsapp"),
            PlatformOrigin::Site => write!(f, "site"),
            PlatformOrigin::AppIos => write!(f, "app_ios"),
            PlatformOrigin::AppAndroid => write!(f, "app_android"),
        }
    }
}

impl std::str::FromStr for PlatformOrigin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "whatsapp" => Ok(PlatformOrigin::Whatsapp),
            "site" => Ok(PlatformOrigin::Site),
            "app_ios" => Ok(PlatformOrigin::AppIos),
            "app_android" => Ok(PlatformOrigin::AppAndroid),
            _ => Err(format!("unknown platform origin: {s}")),
        }
    }
}

/// Payment rail state (fiat or on-chain leg).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Created,
    Pending,
    Authorized,
    Captured,
    Settled,
    Refunded,
    Failed,
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentStatus::Created => write!(f, "created"),
            PaymentStatus::Pending => write!(f, "pending"),
            PaymentStatus::Authorized => write!(f, "authorized"),
            PaymentStatus::Captured => write!(f, "captured"),
            PaymentStatus::Settled => write!(f, "settled"),
            PaymentStatus::Refunded => write!(f, "refunded"),
            PaymentStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Escrow / custody bucket state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustodyStatus {
    PendingLock,
    Locked,
    AccruingYield,
    Releasing,
    Released,
    Disputed,
}

impl fmt::Display for CustodyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CustodyStatus::PendingLock => write!(f, "pending_lock"),
            CustodyStatus::Locked => write!(f, "locked"),
            CustodyStatus::AccruingYield => write!(f, "accruing_yield"),
            CustodyStatus::Releasing => write!(f, "releasing"),
            CustodyStatus::Released => write!(f, "released"),
            CustodyStatus::Disputed => write!(f, "disputed"),
        }
    }
}

/// Dispute workflow state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    Open,
    UnderReview,
    AwaitingEvidence,
    ResolvedBuyer,
    ResolvedSeller,
    Closed,
}

impl fmt::Display for DisputeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisputeStatus::Open => write!(f, "open"),
            DisputeStatus::UnderReview => write!(f, "under_review"),
            DisputeStatus::AwaitingEvidence => write!(f, "awaiting_evidence"),
            DisputeStatus::ResolvedBuyer => write!(f, "resolved_buyer"),
            DisputeStatus::ResolvedSeller => write!(f, "resolved_seller"),
            DisputeStatus::Closed => write!(f, "closed"),
        }
    }
}
