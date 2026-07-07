use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;
use rand::Rng;
use uuid::Uuid;

use crate::models::{
    AddEventRequest, CreateTrackerRequest, PresetStep, Tracker, TrackingEvent, TrackingInfo,
    TrackingStatus, PRESET_STEPS,
};

#[derive(Default)]
pub struct TrackerStore {
    inner: RwLock<HashMap<String, Tracker>>,
}

impl TrackerStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<Tracker> {
        let guard = self.inner.read().expect("tracker store poisoned");
        let mut items: Vec<Tracker> = guard.values().cloned().collect();
        items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        items.into_iter().map(enrich_tracker).collect()
    }

    pub fn get(&self, code: &str) -> Option<Tracker> {
        let guard = self.inner.read().expect("tracker store poisoned");
        guard.get(&code.to_uppercase()).cloned().map(enrich_tracker)
    }

    pub fn create(&self, req: CreateTrackerRequest) -> Tracker {
        let explicit_code = req
            .tracking_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_uppercase());
        let code = explicit_code.unwrap_or_else(generate_tracking_code);

        // Idempotente: se já existe (ex.: vendedor reenviou o mesmo código do site), devolve
        // o tracker existente sem apagar eventos/progresso já registrados.
        if let Some(existing) = self.inner.read().expect("tracker store poisoned").get(&code) {
            return enrich_tracker(existing.clone());
        }

        let now = Utc::now();
        let tracker = Tracker {
            id: Uuid::new_v4(),
            tracking_code: code.clone(),
            description: req.description,
            order_id: req.order_id.filter(|s| !s.trim().is_empty()),
            origin_city: req.origin_city,
            destination_city: req.destination_city,
            seller_phone: req.seller_phone.filter(|s| !s.trim().is_empty()),
            buyer_phone: req.buyer_phone.filter(|s| !s.trim().is_empty()),
            events: vec![],
            created_at: now,
            next_preset_index: 0,
        };
        self.inner
            .write()
            .expect("tracker store poisoned")
            .insert(code, tracker.clone());
        tracker
    }

    pub fn add_event(&self, code: &str, req: AddEventRequest) -> Option<Tracker> {
        let key = code.to_uppercase();
        let mut guard = self.inner.write().expect("tracker store poisoned");
        let tracker = guard.get_mut(&key)?;
        let status = req.status.unwrap_or(TrackingStatus::InTransit);
        tracker.events.push(TrackingEvent {
            id: Uuid::new_v4(),
            status,
            description: req.description,
            location: req.location,
            occurred_at: Utc::now(),
            preset_key: None,
        });
        Some(enrich_tracker(tracker.clone()))
    }

    pub fn add_preset(&self, code: &str, preset_key: &str) -> Result<(Tracker, String), PresetError> {
        let preset = PRESET_STEPS
            .iter()
            .find(|p| p.key == preset_key)
            .ok_or(PresetError::NotFound)?;

        let preset_index = PRESET_STEPS
            .iter()
            .position(|p| p.key == preset_key)
            .ok_or(PresetError::NotFound)?;

        let key = code.to_uppercase();
        let (description, step_label, expected_index) = {
            let guard = self.inner.read().expect("tracker store poisoned");
            let tracker = guard.get(&key).ok_or(PresetError::TrackerNotFound)?;
            (
                personalize_description(tracker, preset),
                preset.label.to_string(),
                preset_events_count(tracker),
            )
        };

        if preset_index != expected_index {
            return Err(PresetError::OutOfOrder {
                expected: PRESET_STEPS
                    .get(expected_index)
                    .map(|p| p.key.to_string())
                    .unwrap_or_else(|| "none".into()),
            });
        }

        if expected_index >= PRESET_STEPS.len() {
            return Err(PresetError::AlreadyComplete);
        }

        let mut guard = self.inner.write().expect("tracker store poisoned");
        let tracker = guard.get_mut(&key).ok_or(PresetError::TrackerNotFound)?;
        tracker.events.push(TrackingEvent {
            id: Uuid::new_v4(),
            status: preset.status.clone(),
            description: description.clone(),
            location: preset.location.map(str::to_string),
            occurred_at: Utc::now(),
            preset_key: Some(preset_key.to_string()),
        });
        Ok((enrich_tracker(tracker.clone()), step_label))
    }

    pub fn to_tracking_info(tracker: &Tracker) -> TrackingInfo {
        let current_status = tracker
            .events
            .last()
            .map(|e| e.status.clone())
            .unwrap_or(TrackingStatus::Unknown);

        TrackingInfo {
            tracking_code: tracker.tracking_code.clone(),
            carrier: "HoldFy Simulador".to_string(),
            current_status,
            events: tracker
                .events
                .iter()
                .rev()
                .map(|e| crate::models::TrackingEventResponse {
                    status: e.status.clone(),
                    description: e.description.clone(),
                    location: e.location.clone(),
                    occurred_at: e.occurred_at,
                })
                .collect(),
            estimated_delivery: None,
            provider_used: "logistica_holdfy".to_string(),
        }
    }
}

fn preset_events_count(tracker: &Tracker) -> usize {
    tracker
        .events
        .iter()
        .filter(|e| e.preset_key.is_some())
        .count()
}

fn enrich_tracker(mut tracker: Tracker) -> Tracker {
    tracker.next_preset_index = preset_events_count(&tracker);
    tracker
}

#[derive(Debug)]
pub enum PresetError {
    NotFound,
    TrackerNotFound,
    OutOfOrder { expected: String },
    AlreadyComplete,
}

fn generate_tracking_code() -> String {
    let mut rng = rand::rng();
    let mut code = String::with_capacity(13);
    for _ in 0..2 {
        let idx = rng.random_range(0..26);
        code.push((b'A' + idx as u8) as char);
    }
    for _ in 0..9 {
        code.push(char::from(b'0' + rng.random_range(0..10)));
    }
    for _ in 0..2 {
        let idx = rng.random_range(0..26);
        code.push((b'A' + idx as u8) as char);
    }
    code.push_str("BR");
    code
}

fn personalize_description(tracker: &Tracker, preset: &PresetStep) -> String {
    let dest = tracker
        .destination_city
        .as_deref()
        .unwrap_or("Rio de Janeiro");
    let origin = tracker.origin_city.as_deref().unwrap_or("cidade de origem");

    match preset.key {
        "left_origin_to_destination" => {
            format!("Produto saiu de {origin} rumo a {dest}")
        }
        "arrived_destination_city" => format!("Produto chegou a {dest}"),
        _ => preset.description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateTrackerRequest;

    #[test]
    fn preset_steps_are_sequential() {
        let store = TrackerStore::new();
        let t = store.create(CreateTrackerRequest {
            description: None,
            order_id: None,
            origin_city: None,
            destination_city: None,
            seller_phone: None,
            buyer_phone: None,
        });
        let code = t.tracking_code.clone();

        assert!(store.add_preset(&code, "left_origin_to_destination").is_err());
        store.add_preset(&code, "distribution_center").unwrap();
        store.add_preset(&code, "left_origin_to_destination").unwrap();
        assert!(store.add_preset(&code, "distribution_center").is_err());
    }
}
