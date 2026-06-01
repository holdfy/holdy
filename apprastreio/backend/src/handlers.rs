use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::json;

use crate::models::{
    AddEventRequest, CreateTrackerRequest, Tracker, TrackingInfo, TrackingStatus, PRESET_STEPS,
};
use crate::notify::WhatsAppNotifier;
use crate::store::{PresetError, TrackerStore};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<TrackerStore>,
    pub notifier: Arc<WhatsAppNotifier>,
}

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "logistica-holdfy-backend",
        "version": "0.1.0"
    }))
}

pub async fn list_presets() -> Json<Vec<PresetStepInfo>> {
    Json(
        PRESET_STEPS
            .iter()
            .enumerate()
            .map(|(index, p)| PresetStepInfo {
                index,
                key: p.key.to_string(),
                label: p.label.to_string(),
                description: p.description.to_string(),
                status: p.status.clone(),
            })
            .collect(),
    )
}

#[derive(serde::Serialize)]
pub struct PresetStepInfo {
    pub index: usize,
    pub key: String,
    pub label: String,
    pub description: String,
    pub status: crate::models::TrackingStatus,
}

pub async fn list_trackers(State(state): State<AppState>) -> Json<Vec<Tracker>> {
    Json(state.store.list())
}

pub async fn create_tracker(
    State(state): State<AppState>,
    Json(req): Json<CreateTrackerRequest>,
) -> (StatusCode, Json<Tracker>) {
    let tracker = state.store.create(req);
    (StatusCode::CREATED, Json(tracker))
}

pub async fn get_tracker(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<Tracker>, StatusCode> {
    state
        .store
        .get(&code)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn track_shipment(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<TrackingInfo>, StatusCode> {
    let tracker = state.store.get(&code).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(TrackerStore::to_tracking_info(&tracker)))
}

pub async fn add_event(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(req): Json<AddEventRequest>,
) -> Result<Json<Tracker>, StatusCode> {
    let tracker = state.store.add_event(&code, req.clone()).ok_or(StatusCode::NOT_FOUND)?;
    let label = req
        .status
        .as_ref()
        .map(TrackingStatus::label)
        .unwrap_or("Atualização");
    notify_tracker_step(&state, &tracker, label, &req.description);
    Ok(Json(tracker))
}

#[derive(serde::Deserialize)]
pub struct AddPresetRequest {
    pub preset_key: String,
}

pub async fn add_preset_step(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(req): Json<AddPresetRequest>,
) -> Result<Json<Tracker>, (StatusCode, Json<serde_json::Value>)> {
    match state.store.add_preset(&code, &req.preset_key) {
        Ok((tracker, step_label)) => {
            let description = tracker
                .events
                .last()
                .map(|e| e.description.as_str())
                .unwrap_or("");
            notify_tracker_step(&state, &tracker, &step_label, description);
            Ok(Json(tracker))
        }
        Err(PresetError::NotFound) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "preset_key inválido" })),
        )),
        Err(PresetError::TrackerNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "código de rastreio não encontrado" })),
        )),
        Err(PresetError::OutOfOrder { expected }) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "etapa fora de ordem — avance a sequência",
                "expected_preset_key": expected
            })),
        )),
        Err(PresetError::AlreadyComplete) => Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "todas as etapas já foram concluídas" })),
        )),
    }
}

fn notify_tracker_step(state: &AppState, tracker: &Tracker, step_label: &str, description: &str) {
    let seller = tracker.seller_phone.clone().unwrap_or_default();
    state.notifier.spawn_notify(
        seller,
        tracker.order_id.clone(),
        tracker.tracking_code.clone(),
        step_label.to_string(),
        description.to_string(),
    );
}
