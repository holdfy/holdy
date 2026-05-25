//! Handlers para cotação, geração de etiqueta e rastreamento.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use apicash_logistics::{
    PackageDimensions, ShippingAddress, ShippingLabel, ShippingQuote, ShippingQuoteRequest,
    TrackingInfo,
};

use crate::error::ApiError;
use crate::state::AppState;

// ─── Quote ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct QuoteRequest {
    pub from_postal_code: String,
    pub to_postal_code: String,
    /// Peso em kg.
    pub weight_kg: String,
    pub width_cm: u32,
    pub height_cm: u32,
    pub length_cm: u32,
}

#[derive(Serialize)]
pub struct QuoteResponse {
    pub quotes: Vec<ShippingQuote>,
}

/// `POST /logistics/shipping/quote`
pub async fn quote_shipping(
    State(state): State<Arc<AppState>>,
    Json(req): Json<QuoteRequest>,
) -> Result<Json<QuoteResponse>, ApiError> {
    let weight_kg = req
        .weight_kg
        .parse()
        .map_err(|_| ApiError::bad_request("weight_kg inválido"))?;

    let shipping_req = ShippingQuoteRequest {
        from_postal_code: req.from_postal_code,
        to_postal_code: req.to_postal_code,
        package: PackageDimensions {
            weight_kg,
            width_cm: req.width_cm,
            height_cm: req.height_cm,
            length_cm: req.length_cm,
        },
    };

    let quotes = state
        .logistics
        .quote(&shipping_req)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(QuoteResponse { quotes }))
}

// ─── Label ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LabelRequest {
    pub order_id: String,
    pub from: ShippingAddress,
    pub to: ShippingAddress,
    pub carrier: String,
    pub weight_kg: String,
    pub width_cm: u32,
    pub height_cm: u32,
    pub length_cm: u32,
}

/// `POST /logistics/shipping/label`
pub async fn generate_label(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LabelRequest>,
) -> Result<Json<ShippingLabel>, ApiError> {
    let carrier = parse_carrier(&req.carrier)?;
    let weight_kg = req
        .weight_kg
        .parse()
        .map_err(|_| ApiError::bad_request("weight_kg inválido"))?;

    let label = state
        .logistics
        .generate_label(
            &req.order_id,
            &req.from,
            &req.to,
            &carrier,
            &PackageDimensions {
                weight_kg,
                width_cm: req.width_cm,
                height_cm: req.height_cm,
                length_cm: req.length_cm,
            },
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    Ok(Json(label))
}

// ─── Tracking ────────────────────────────────────────────────────────────────

/// `GET /logistics/tracking/:code`
pub async fn track_shipment(
    State(state): State<Arc<AppState>>,
    Path(code): Path<String>,
) -> Result<Json<TrackingInfo>, ApiError> {
    let info = state
        .logistics
        .track(&code)
        .await
        .map_err(|e| match e {
            apicash_logistics::LogisticsError::TrackingNotFound(_) => {
                ApiError::not_found("código de rastreio não encontrado")
            }
            other => ApiError::internal(other.to_string()),
        })?;

    Ok(Json(info))
}

fn parse_carrier(s: &str) -> Result<apicash_logistics::CarrierCode, ApiError> {
    match s.to_lowercase().as_str() {
        "correios_pac" | "correios" => Ok(apicash_logistics::CarrierCode::CorreiosPac),
        "correios_sedex" | "sedex" => Ok(apicash_logistics::CarrierCode::CorreiosSedex),
        "jadlog_package" | "jadlog" => Ok(apicash_logistics::CarrierCode::JadlogPackage),
        "jadlog_com" => Ok(apicash_logistics::CarrierCode::JadlogCom),
        _ => Err(ApiError::bad_request(format!("transportadora '{s}' não reconhecida"))),
    }
}
