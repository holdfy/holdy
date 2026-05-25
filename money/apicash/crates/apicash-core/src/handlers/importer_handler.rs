//! Handler for `POST /v1/listings/import`.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use apicash_importer::ProductDraft;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ImportRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct ImportResponse {
    pub title: String,
    pub description: Option<String>,
    pub price_suggested: Option<String>,
    pub photos: Vec<String>,
    pub source_url: String,
    pub source_platform: String,
    pub extractor_used: String,
}

impl From<ProductDraft> for ImportResponse {
    fn from(d: ProductDraft) -> Self {
        Self {
            title: d.title,
            description: d.description,
            price_suggested: d.price_suggested.map(|p| p.to_string()),
            photos: d.photos,
            source_url: d.source_url,
            source_platform: format!("{:?}", d.source_platform).to_lowercase(),
            extractor_used: d.extractor_used,
        }
    }
}

/// `POST /v1/listings/import` — import a product from any URL.
pub async fn import_listing(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, ApiError> {
    if req.url.trim().is_empty() {
        return Err(ApiError::bad_request("url não pode ser vazio"));
    }

    let draft = state
        .importer
        .import(&req.url)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    Ok(Json(ImportResponse::from(draft)))
}
