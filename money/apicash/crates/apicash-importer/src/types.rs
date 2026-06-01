//! Domain types for imported products.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// All platforms supported by the importer.
/// Três fluxos (API, WhatsApp, site) usam o mesmo `SourcePlatform`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePlatform {
    MercadoLivre,
    Olx,
    Shopee,
    Instagram,
    Facebook,
    TikTok,
    WhatsAppBusiness,
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
        } else if lower.contains("wa.me") || lower.contains("api.whatsapp.com") {
            Self::WhatsAppBusiness
        } else {
            Self::Generic
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::MercadoLivre => "Mercado Livre",
            Self::Olx => "OLX",
            Self::Shopee => "Shopee",
            Self::Instagram => "Instagram",
            Self::Facebook => "Facebook Marketplace",
            Self::TikTok => "TikTok Shop",
            Self::WhatsAppBusiness => "WhatsApp Business",
            Self::Generic => "E-commerce",
            Self::Unknown => "Desconhecido",
        }
    }
}

/// Full product data scraped from an external listing URL.
///
/// Usado pelos três fluxos: API (`POST /v1/listings/import`), WhatsApp (URL no chat), Site (dialog de importação).
/// Salvo no PostgreSQL via `ListingRepository`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDraft {
    pub title: String,
    pub description: Option<String>,
    pub price_suggested: Option<Decimal>,
    /// MinIO URLs após re-hosting (ou URLs externas originais se MinIO não configurado).
    pub photos: Vec<String>,
    pub source_url: String,
    pub source_platform: SourcePlatform,
    pub extractor_used: String,
    // --- Campos estruturados adicionais ---
    /// Garantia do produto (ex.: "12 meses de fábrica").
    pub guarantee: Option<String>,
    /// Condição: "new" | "used" | "refurbished".
    pub condition: Option<String>,
    /// Localização do vendedor (cidade/estado).
    pub location: Option<String>,
    /// Nome do vendedor na plataforma de origem.
    pub seller_name: Option<String>,
    /// Rating/reputação do vendedor (ex.: "4.8/5" ou "Platina").
    pub seller_rating: Option<String>,
    /// URL de vídeo do anúncio (og:video para Instagram/TikTok/etc).
    pub video_url: Option<String>,
    /// Todos os atributos extras em formato livre (JSONB no postgres).
    pub raw_attributes: serde_json::Value,
}
