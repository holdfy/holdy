//! Domain types for shipping operations.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CarrierCode {
    Correios,
    Jadlog,
    CorreiosPac,
    CorreiosSedex,
    JadlogPackage,
    JadlogCom,
}

impl CarrierCode {
    pub fn melhor_envio_id(&self) -> u32 {
        match self {
            CarrierCode::CorreiosPac => 1,
            CarrierCode::CorreiosSedex => 2,
            CarrierCode::JadlogPackage => 3,
            CarrierCode::JadlogCom => 4,
            CarrierCode::Correios => 1,
            CarrierCode::Jadlog => 3,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CarrierCode::CorreiosPac | CarrierCode::Correios => "Correios PAC",
            CarrierCode::CorreiosSedex => "Correios SEDEX",
            CarrierCode::JadlogPackage | CarrierCode::Jadlog => "Jadlog Package",
            CarrierCode::JadlogCom => "Jadlog .COM",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingAddress {
    pub name: String,
    pub cpf_cnpj: String,
    pub postal_code: String,
    pub address: String,
    pub number: String,
    pub complement: Option<String>,
    pub district: String,
    pub city: String,
    pub state_abbr: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDimensions {
    /// Peso em kg.
    pub weight_kg: Decimal,
    /// Dimensões em cm.
    pub width_cm: u32,
    pub height_cm: u32,
    pub length_cm: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingQuoteRequest {
    pub from_postal_code: String,
    pub to_postal_code: String,
    pub package: PackageDimensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingQuote {
    pub carrier: CarrierCode,
    pub carrier_label: String,
    pub service_name: String,
    pub price_brl: Decimal,
    pub estimated_days: u32,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingLabel {
    pub tracking_code: String,
    pub carrier: CarrierCode,
    /// URL do PDF da etiqueta de envio.
    pub label_url: String,
    pub order_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackingStatus {
    Posted,
    InTransit,
    OutForDelivery,
    Delivered,
    ReturnInProgress,
    Returned,
    Exception,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub status: TrackingStatus,
    pub description: String,
    pub location: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingInfo {
    pub tracking_code: String,
    pub carrier: String,
    pub current_status: TrackingStatus,
    pub events: Vec<TrackingEvent>,
    pub estimated_delivery: Option<DateTime<Utc>>,
    #[serde(default)]
    pub provider_used: String,
}
