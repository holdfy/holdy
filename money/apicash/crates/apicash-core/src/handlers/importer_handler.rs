//! Handler for `POST /v1/listings/import` (sync) and `POST /v1/listings/import/async`.

use std::sync::Arc;

use apicash_auth::JwtClaims;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use apicash_importer::ProductDraft;

use crate::activity_store::spawn_record_import;
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
    pub video_url: Option<String>,
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
            video_url: d.video_url,
            raw_attributes: d.raw_attributes,
        }
    }
}

// ─── Async import ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AsyncImportResponse {
    pub job_id: String,
    pub status: &'static str,
    pub poll_url: String,
}

#[derive(Serialize)]
pub struct ImportJobResponse {
    pub job_id: String,
    pub status: String,
    pub listing_id: Option<String>,
    pub error_msg: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    /// Dados completos do listing (preenchido quando `status == "done"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing: Option<ImportResponse>,
}

/// `POST /v1/listings/import/async` — enfileira importação no Pulsar e retorna imediatamente.
pub async fn import_listing_async(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<AsyncImportResponse>, ApiError> {
    if req.url.trim().is_empty() {
        return Err(ApiError::bad_request("url não pode ser vazio"));
    }

    let effective_user_id = claims
        .map(|Extension(c)| c.current_user_id())
        .or(req.user_id);

    let Some(producer) = &state.event_producer else {
        return Err(ApiError::bad_request(
            "fila assíncrona não configurada (APICASH_PULSAR__SERVICE_URL ausente); use POST /v1/listings/import",
        ));
    };

    let Some(repo) = &state.listing_repo else {
        return Err(ApiError::bad_request(
            "banco de dados não configurado (DATABASE_URL ausente); use POST /v1/listings/import",
        ));
    };

    let job_id = repo
        .create_import_job(&req.url, effective_user_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    {
        let mut p = producer.lock().await;
        p.publish_import_requested(apicash_events::ImportRequestedEvent {
            job_id,
            url: req.url.clone(),
            user_id: effective_user_id,
            requested_at: chrono::Utc::now(),
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    }

    Ok(Json(AsyncImportResponse {
        job_id: job_id.to_string(),
        status: "queued",
        poll_url: format!("/v1/listings/jobs/{job_id}"),
    }))
}

/// `GET /v1/listings/jobs/:id` — consulta status de um job de importação.
pub async fn get_import_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ImportJobResponse>, ApiError> {
    let Some(repo) = &state.listing_repo else {
        return Err(ApiError::bad_request("banco de dados não configurado"));
    };

    let job = repo
        .get_import_job(id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
        .ok_or_else(|| ApiError::not_found("job não encontrado"))?;

    // Inclui dados do listing quando concluído
    let listing = if job.status == "done" {
        if let Some(lid) = job.listing_id {
            repo.get_listing(lid)
                .await
                .ok()
                .flatten()
                .map(|l| ImportResponse {
                    listing_id: Some(lid),
                    title: l.title,
                    description: l.description,
                    price_suggested: l.price_suggested,
                    photos: l.photos,
                    source_url: l.source_url,
                    source_platform: l.source_platform,
                    extractor_used: l.extractor_used,
                    guarantee: None,
                    condition: None,
                    location: None,
                    seller_name: l.seller_name,
                    seller_rating: l.seller_rating,
                    raw_attributes: serde_json::Value::Object(Default::default()),
                    video_url: None,
                })
        } else {
            None
        }
    } else {
        None
    };

    Ok(Json(ImportJobResponse {
        job_id: job.id.to_string(),
        status: job.status,
        listing_id: job.listing_id.map(|u| u.to_string()),
        error_msg: job.error_msg,
        queued_at: job.queued_at,
        completed_at: job.completed_at,
        listing,
    }))
}

/// `PATCH /v1/listings/:id/order` — vincula um listing a um pedido existente.
pub async fn link_listing_to_order(
    State(state): State<Arc<AppState>>,
    Path(listing_id): Path<Uuid>,
    Json(body): Json<LinkOrderBody>,
) -> Result<axum::http::StatusCode, ApiError> {
    let Some(repo) = &state.listing_repo else {
        return Err(ApiError::bad_request("banco de dados não configurado"));
    };
    repo.set_order_id(listing_id, body.order_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct LinkOrderBody {
    pub order_id: Uuid,
}

/// `POST /v1/listings/import` — faz scraping da URL e persiste no Postgres.
pub async fn import_listing(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Json(req): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, ApiError> {
    if req.url.trim().is_empty() {
        return Err(ApiError::bad_request("url não pode ser vazio"));
    }

    // user_id: JWT tem prioridade (site), fallback para body (WhatsApp via API key)
    let effective_user_id = claims
        .map(|Extension(c)| c.current_user_id())
        .or(req.user_id);

    let draft = state
        .importer
        .import(&req.url)
        .await
        .map_err(|e| ApiError::bad_request(e.to_string()))?;

    let mut resp = ImportResponse::from(draft.clone());

    if let Some(repo) = &state.listing_repo {
        match repo.save(&draft, effective_user_id, None).await {
            Ok(id) => {
                tracing::info!(listing_id = %id, platform = ?draft.source_platform, user_id = ?effective_user_id, "listing saved to postgres");
                resp.listing_id = Some(id);
            }
            Err(e) => {
                tracing::warn!(error = %e, "listing: falha ao salvar no postgres (continuando)");
            }
        }
    }

    // Log no MongoDB — paridade com WhatsApp (wa_messages registra a URL inbound).
    spawn_record_import(
        state.activity_store.clone(),
        effective_user_id,
        resp.listing_id,
        &resp.title,
        &resp.source_platform,
        resp.photos.len(),
        resp.price_suggested.clone(),
        &req.url,
    );

    Ok(Json(resp))
}
