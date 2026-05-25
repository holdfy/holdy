//! Handler for `POST /v1/listings/import`.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use apicash_importer::ProductDraft;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ImportRequest {
    pub url: String,
    /// Opcional — usuário autenticado que disparou a importação.
    pub user_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ImportResponse {
    /// UUID gerado no PostgreSQL (None se DB não configurado).
    pub listing_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub price_suggested: Option<String>,
    pub photos: Vec<String>,
    pub source_url: String,
    pub source_platform: String,
    pub extractor_used: String,
    pub guarantee: Option<String>,
    pub condition: Option<String>,
    pub location: Option<String>,
    pub seller_name: Option<String>,
    pub seller_rating: Option<String>,
    pub raw_attributes: serde_json::Value,
}

impl From<ProductDraft> for ImportResponse {
    fn from(d: ProductDraft) -> Self {
        Self {
            listing_id: None,
            title: d.title,
            description: d.description,
            price_suggested: d.price_suggested.map(|p| p.to_string()),
            photos: d.photos,
            source_url: d.source_url,
            source_platform: format!("{:?}", d.source_platform).to_ascii_lowercase(),
            extractor_used: d.extractor_used,
            guarantee: d.guarantee,
            condition: d.condition,
            location: d.location,
            seller_name: d.seller_name,
            seller_rating: d.seller_rating,
            raw_attributes: d.raw_attributes,
        }
    }
}

/// `POST /v1/listings/import` — faz scraping da URL e persiste no Postgres.
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

    let mut resp = ImportResponse::from(draft.clone());

    if let Some(repo) = &state.listing_repo {
        match repo.save(&draft, req.user_id, None).await {
            Ok(id) => {
                tracing::info!(listing_id = %id, platform = ?draft.source_platform, "listing saved to postgres");
                resp.listing_id = Some(id);
            }
            Err(e) => {
                tracing::warn!(error = %e, "listing: falha ao salvar no postgres (continuando)");
            }
        }
    }

    Ok(Json(resp))
}
