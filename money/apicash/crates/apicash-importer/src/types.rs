//! Domain types for imported products.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Platform from which the product was imported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePlatform {
    MercadoLivre,
    Olx,
    Shopee,
    Instagram,
    Facebook,
    TikTok,
    Generic,
    Unknown,
}

impl SourcePlatform {
    pub fn detect(url: &str) -> Self {
        let lower = url.to_lowercase();
        if lower.contains("mercadolivre") || lower.contains("mercadolibre") {
            Self::MercadoLivre
        } else if lower.contains("olx.com") {
            Self::Olx
        } else if lower.contains("shopee") {
            Self::Shopee
        } else if lower.contains("instagram.com") {
            Self::Instagram
        } else if lower.contains("facebook.com") {
            Self::Facebook
        } else if lower.contains("tiktok.com") {
            Self::TikTok
        } else {
            Self::Generic
        }
    }
}

/// Product data extracted from a URL — ready to create a listing proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDraft {
    pub title: String,
    pub description: Option<String>,
    /// Suggested price in BRL, extracted from the source.
    pub price_suggested: Option<Decimal>,
    /// Remote image URLs (caller should re-host these before saving).
    pub photos: Vec<String>,
    pub source_url: String,
    pub source_platform: SourcePlatform,
    /// Which extractor produced this draft.
    pub extractor_used: String,
}
