use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl TrackingStatus {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Posted => "Postado",
            Self::InTransit => "Em trânsito",
            Self::OutForDelivery => "Saiu para entrega",
            Self::Delivered => "Entregue",
            Self::ReturnInProgress => "Retorno em andamento",
            Self::Returned => "Devolvido",
            Self::Exception => "Problema na entrega",
            Self::Unknown => "Status desconhecido",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    pub id: Uuid,
    pub status: TrackingStatus,
    pub description: String,
    pub location: Option<String>,
    pub occurred_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tracker {
    pub id: Uuid,
    pub tracking_code: String,
    pub description: Option<String>,
    pub order_id: Option<String>,
    pub origin_city: Option<String>,
    pub destination_city: Option<String>,
    pub seller_phone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buyer_phone: Option<String>,
    pub events: Vec<TrackingEvent>,
    pub created_at: DateTime<Utc>,
    /// Índice (0-based) da próxima etapa preset permitida (= events.len()).
    pub next_preset_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingInfo {
    pub tracking_code: String,
    pub carrier: String,
    pub current_status: TrackingStatus,
    pub events: Vec<TrackingEventResponse>,
    pub estimated_delivery: Option<DateTime<Utc>>,
    #[serde(default)]
    pub provider_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEventResponse {
    pub status: TrackingStatus,
    pub description: String,
    pub location: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrackerRequest {
    pub description: Option<String>,
    pub order_id: Option<String>,
    pub origin_city: Option<String>,
    pub destination_city: Option<String>,
    pub seller_phone: Option<String>,
    #[serde(default)]
    pub buyer_phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddEventRequest {
    pub description: String,
    pub status: Option<TrackingStatus>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PresetStep {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub status: TrackingStatus,
    pub location: Option<&'static str>,
}

pub const PRESET_STEPS: &[PresetStep] = &[
    PresetStep {
        key: "distribution_center",
        label: "Centro de distribuição",
        description: "Produto chegou no centro de distribuição",
        status: TrackingStatus::InTransit,
        location: Some("Centro de Distribuição"),
    },
    PresetStep {
        key: "left_origin_to_destination",
        label: "Saiu rumo ao destino",
        description: "Produto saiu da cidade de origem rumo ao Rio de Janeiro",
        status: TrackingStatus::InTransit,
        location: Some("Em trânsito"),
    },
    PresetStep {
        key: "arrived_destination_city",
        label: "Chegou ao destino",
        description: "Produto chegou ao Rio de Janeiro",
        status: TrackingStatus::InTransit,
        location: Some("Rio de Janeiro - RJ"),
    },
    PresetStep {
        key: "out_for_delivery",
        label: "Saiu para entrega",
        description: "Produto saiu para entrega",
        status: TrackingStatus::OutForDelivery,
        location: Some("Unidade de entrega"),
    },
    PresetStep {
        key: "delivered",
        label: "Entregue",
        description: "Produto foi entregue",
        status: TrackingStatus::Delivered,
        location: Some("Destinatário"),
    },
];
