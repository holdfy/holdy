//! LogisticsService: cotação, geração de etiqueta e rastreamento multi-provedor.

use rust_decimal::Decimal;
use std::str::FromStr;

use crate::client::MelhorEnvioClient;
use crate::error::LogisticsError;
use crate::tracking::CascadingTracker;
use crate::types::{
    CarrierCode, PackageDimensions, ShippingAddress, ShippingLabel, ShippingQuote,
    ShippingQuoteRequest, TrackingInfo,
};

pub struct LogisticsService {
    client: MelhorEnvioClient,
    tracker: CascadingTracker,
}

impl LogisticsService {
    pub fn new(client: MelhorEnvioClient) -> Self {
        let tracker = CascadingTracker::from_env(client.clone());
        Self { client, tracker }
    }

    pub fn from_env() -> Result<Self, LogisticsError> {
        Ok(Self::new(MelhorEnvioClient::from_env()?))
    }

    /// Retorna cotações para os principais transportadores.
    pub async fn quote(
        &self,
        req: &ShippingQuoteRequest,
    ) -> Result<Vec<ShippingQuote>, LogisticsError> {
        let body = serde_json::json!({
            "from": { "postal_code": req.from_postal_code },
            "to":   { "postal_code": req.to_postal_code },
            "package": {
                "weight": req.package.weight_kg,
                "width":  req.package.width_cm,
                "height": req.package.height_cm,
                "length": req.package.length_cm,
            },
            "services": "1,2,3,4" // PAC, SEDEX, Jadlog Package, Jadlog .COM
        });

        let resp = self.client.post_json("/me/shipment/calculate", &body).await?;

        let quotes = resp
            .as_array()
            .ok_or_else(|| LogisticsError::ApiError("resposta inesperada".into()))?
            .iter()
            .filter_map(|item| parse_quote(item))
            .collect();

        Ok(quotes)
    }

    /// Gera etiqueta de envio e retorna o código de rastreio + URL do PDF.
    pub async fn generate_label(
        &self,
        order_id: &str,
        from: &ShippingAddress,
        to: &ShippingAddress,
        carrier: &CarrierCode,
        package: &PackageDimensions,
    ) -> Result<ShippingLabel, LogisticsError> {
        // Step 1: add to cart
        let cart_body = serde_json::json!({
            "service": carrier.melhor_envio_id(),
            "agency": null,
            "from": address_payload(from),
            "to":   address_payload(to),
            "products": [{
                "name": "Produto",
                "quantity": 1,
                "unitary_value": 1
            }],
            "volumes": [{
                "height": package.height_cm,
                "width":  package.width_cm,
                "length": package.length_cm,
                "weight": package.weight_kg,
            }],
            "options": {
                "insurance_value": 0,
                "receipt": false,
                "own_hand": false,
                "tags": [{ "tag": order_id, "url": null }]
            }
        });

        let cart_resp = self.client.post_json("/me/cart", &cart_body).await?;

        let cart_id = cart_resp
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LogisticsError::ApiError("id ausente no carrinho".into()))?
            .to_string();

        // Step 2: checkout (purchase)
        let checkout_body = serde_json::json!({ "orders": [cart_id] });
        self.client.post_json("/me/shipment/checkout", &checkout_body).await?;

        // Step 3: generate label
        let label_body = serde_json::json!({ "orders": [cart_id] });
        let label_resp = self.client.post_json("/me/shipment/generate", &label_body).await?;

        let label_url = label_resp
            .get(0)
            .and_then(|o| o.get("link"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tracking_code = label_resp
            .get(0)
            .and_then(|o| o.get("tracking"))
            .and_then(|v| v.as_str())
            .unwrap_or(&cart_id)
            .to_string();

        Ok(ShippingLabel {
            tracking_code,
            carrier: carrier.clone(),
            label_url,
            order_id: order_id.to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Rastreia uma encomenda pelo código de rastreio (multi-provedor com circuit breaker).
    pub async fn track(&self, tracking_code: &str) -> Result<TrackingInfo, LogisticsError> {
        self.tracker.track(tracking_code).await
    }
}

fn address_payload(addr: &ShippingAddress) -> serde_json::Value {
    serde_json::json!({
        "name":        addr.name,
        "phone":       addr.phone,
        "email":       addr.email,
        "document":    addr.cpf_cnpj,
        "address":     addr.address,
        "number":      addr.number,
        "complement":  addr.complement,
        "district":    addr.district,
        "city":        addr.city,
        "state_abbr":  addr.state_abbr,
        "country_id":  "BR",
        "postal_code": addr.postal_code,
    })
}

fn parse_quote(item: &serde_json::Value) -> Option<ShippingQuote> {
    let service_name = item.get("name")?.as_str()?.to_string();
    let price_str = item.get("price")?.as_str()?;
    let price_brl = Decimal::from_str(price_str).ok()?;
    let estimated_days = item
        .get("delivery_time")
        .and_then(|v| v.as_u64())
        .unwrap_or(7) as u32;

    let company_name = item
        .get("company")
        .and_then(|c| c.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Transportadora");

    let carrier = if company_name.to_lowercase().contains("correio") {
        if service_name.to_lowercase().contains("sedex") {
            CarrierCode::CorreiosSedex
        } else {
            CarrierCode::CorreiosPac
        }
    } else {
        if service_name.to_lowercase().contains("com") {
            CarrierCode::JadlogCom
        } else {
            CarrierCode::JadlogPackage
        }
    };

    Some(ShippingQuote {
        carrier_label: format!("{company_name} {service_name}"),
        carrier,
        service_name,
        price_brl,
        estimated_days,
        currency: "BRL".to_string(),
    })
}

