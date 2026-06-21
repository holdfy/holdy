//! Proposal handlers — two-party escrow negotiation (seller creates, buyer accepts/rejects).
//!
//! Flow:
//!   1. Seller: POST /proposals              → proposal_id
//!   2. Buyer:  GET  /proposals/{id}         → proposal details + status
//!   3. Buyer:  POST /proposals/{id}/accept  → order created, returns pix_br_code
//!   4. Buyer:  POST /proposals/{id}/reject  → proposal closed

use std::sync::Arc;

use apicash_auth::JwtClaims;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use chrono::{Duration, Utc};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::dto::{
    AcceptProposalRequest, AcceptProposalResponse, CreateProposalRequest, ProposalResponse,
    ProposalStatus, StoredProposal,
};
use crate::error::ApiError;
use crate::state::AppState;

use super::order_handler::create_escrow_order_core;

const PROPOSAL_TTL_MINUTES: i64 = 60;
const DEFAULT_CPF_PLACEHOLDER: &str = "52998224725";

/// POST /proposals — seller creates a new proposal for a buyer.
/// When `buyer_id` is omitted the proposal is "open" — any authenticated buyer can accept it.
#[instrument(skip(state, claims, req))]
pub async fn create_proposal(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Json(req): Json<CreateProposalRequest>,
) -> Result<(StatusCode, Json<ProposalResponse>), ApiError> {
    req.validate().map_err(ApiError::bad_request)?;

    let buyer_id = req.buyer_id.unwrap_or(Uuid::nil());
    let (seller_id, seller_document) = resolve_seller_id(&state, claims)?;

    if buyer_id != Uuid::nil() && seller_id == buyer_id {
        return Err(ApiError::bad_request(
            "seller and buyer must be different users",
        ));
    }

    let listing_id = req.listing_id;
    let pix_key = req.seller_pix_key.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(String::from);

    let now = Utc::now();
    let proposal = StoredProposal {
        id: Uuid::new_v4(),
        seller_id,
        seller_document,
        buyer_id,
        amount: req.amount.trim().to_string(),
        description: req.description.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        status: ProposalStatus::Pending,
        created_at: now,
        expires_at: now + Duration::minutes(PROPOSAL_TTL_MINUTES),
        order_id: None,
        listing_id,
    };

    let id = proposal.id;
    state
        .proposals
        .save(proposal.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "proposal save failed");
            ApiError::internal("proposal persistence failed")
        })?;

    // Persist seller PIX key so off-ramp fires automatically after delivery confirmation.
    if let (Some(key), Some(repo)) = (&pix_key, &state.listing_repo) {
        if let Err(e) = repo.upsert_pix_key(seller_id, key).await {
            warn!(%seller_id, error = %e, "failed to save seller pix_key (non-critical)");
        } else {
            info!(%seller_id, "seller pix_key saved for auto off-ramp");
        }
    }

    // Persist seller WhatsApp so tracking monitor can send notifications.
    if let (Some(phone), Some(repo)) = (&req.seller_phone, &state.listing_repo) {
        let phone = phone.trim();
        if !phone.is_empty() {
            if let Err(e) = repo.upsert_phone(seller_id, phone).await {
                warn!(%seller_id, error = %e, "failed to save seller phone (non-critical)");
            } else {
                info!(%seller_id, "seller phone saved for tracking notifications");
            }
        }
    }

    info!(%id, %seller_id, %buyer_id, open = buyer_id.is_nil(), "proposal created");

    Ok((StatusCode::CREATED, Json(ProposalResponse::from(&proposal))))
}

/// GET /proposals/{id} — any party may check the proposal status.
#[instrument(skip(state, claims), fields(proposal_id = %id))]
pub async fn get_proposal(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProposalResponse>, ApiError> {
    let proposal = load_proposal(&state, id).await?;

    if !state.auth.config().auth_disabled {
        let actor = require_claims(claims)?;
        let actor_id = actor.current_user_id();
        if actor_id != proposal.seller_id && actor_id != proposal.buyer_id {
            return Err(ApiError::forbidden(
                "only the seller or buyer of this proposal can view it",
            ));
        }
    }

    let mut resp = ProposalResponse::from(&proposal);

    // Attach first listing photo — non-critical, soft failure
    if let (Some(lid), Some(repo)) = (proposal.listing_id, &state.listing_repo) {
        if let Ok(Some(listing)) = repo.get_listing(lid).await {
            resp.listing_photo = listing.photos.into_iter().next();
        }
    }

    Ok(Json(resp))
}

/// POST /proposals/{id}/accept — buyer accepts, creating the escrow order + PIX QR.
#[instrument(skip(state, claims, body), fields(proposal_id = %id))]
pub async fn accept_proposal(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Path(id): Path<Uuid>,
    Json(body): Json<AcceptProposalRequest>,
) -> Result<Json<AcceptProposalResponse>, ApiError> {
    let mut proposal = load_proposal(&state, id).await?;

    if proposal.is_expired() {
        return Err(ApiError::bad_request("proposal has expired"));
    }
    if proposal.status != ProposalStatus::Pending {
        return Err(ApiError::bad_request(format!(
            "proposal is not pending (current status: {})",
            proposal.status
        )));
    }

    let buyer_id = if state.auth.config().auth_disabled {
        proposal.buyer_id
    } else {
        let actor = require_claims(claims)?;
        let actor_id = actor.current_user_id();
        // nil buyer_id = open proposal — any authenticated buyer may accept
        if !proposal.buyer_id.is_nil() && actor_id != proposal.buyer_id {
            warn!(
                %actor_id,
                expected_buyer = %proposal.buyer_id,
                "proposal accept rejected: actor is not the proposal buyer"
            );
            return Err(ApiError::forbidden(
                "only the designated buyer can accept this proposal",
            ));
        }
        actor_id
    };

    let cpf = body
        .cpf
        .as_deref()
        .unwrap_or(DEFAULT_CPF_PLACEHOLDER);
    let social_links: Vec<String> = body.social_links.unwrap_or_default();
    let desc = proposal.description.as_deref();

    let order = create_escrow_order_core(
        &state,
        buyer_id,
        proposal.seller_id,
        &proposal.amount,
        cpf,
        &social_links,
        desc,
    )
    .await?;

    let pix_br_code = order.pix_br_code.clone().unwrap_or_default();
    let funding_instruction = order
        .funding_instruction
        .clone()
        .unwrap_or_else(|| "PIX copia-e-cola disponível em pix_br_code.".to_string());

    proposal.status = ProposalStatus::Accepted;
    proposal.order_id = Some(order.id);
    state.proposals.update(proposal.clone()).await.map_err(|e| {
        error!(error = %e, "proposal update failed");
        ApiError::internal("proposal persistence failed")
    })?;

    // Persist buyer WhatsApp so tracking monitor can send notifications.
    if let (Some(phone), Some(repo)) = (&body.buyer_phone, &state.listing_repo) {
        let phone = phone.trim();
        if !phone.is_empty() {
            if let Err(e) = repo.upsert_phone(buyer_id, phone).await {
                warn!(%buyer_id, error = %e, "failed to save buyer phone (non-critical)");
            } else {
                info!(%buyer_id, "buyer phone saved for tracking notifications");
            }
        }
    }

    // Link imported listing to the newly created order if listing_id was provided.
    if let (Some(listing_id), Some(repo)) = (proposal.listing_id, &state.listing_repo) {
        if let Err(e) = repo.set_order_id(listing_id, order.id).await {
            warn!(%listing_id, order_id = %order.id, error = %e, "listing→order link failed (non-critical)");
        } else {
            info!(%listing_id, order_id = %order.id, "listing linked to order");
        }
    }

    info!(
        proposal_id = %id,
        order_id = %order.id,
        buyer_id = %buyer_id,
        "proposal accepted → order created"
    );

    Ok(Json(AcceptProposalResponse {
        proposal_id: id,
        order_id: order.id,
        pix_br_code,
        amount: proposal.amount,
        status: ProposalStatus::Accepted,
        funding_instruction,
    }))
}

/// POST /proposals/{id}/reject — buyer rejects the proposal.
#[instrument(skip(state, claims), fields(proposal_id = %id))]
pub async fn reject_proposal(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProposalResponse>, ApiError> {
    let mut proposal = load_proposal(&state, id).await?;

    if proposal.status != ProposalStatus::Pending {
        return Err(ApiError::bad_request(format!(
            "proposal is not pending (current status: {})",
            proposal.status
        )));
    }

    if !state.auth.config().auth_disabled {
        let actor = require_claims(claims)?;
        let actor_id = actor.current_user_id();
        if actor_id != proposal.buyer_id {
            return Err(ApiError::forbidden(
                "only the designated buyer can reject this proposal",
            ));
        }
    }

    proposal.status = ProposalStatus::Rejected;
    state.proposals.update(proposal.clone()).await.map_err(|e| {
        error!(error = %e, "proposal update failed");
        ApiError::internal("proposal persistence failed")
    })?;

    info!(proposal_id = %id, "proposal rejected by buyer");

    Ok(Json(ProposalResponse::from(&proposal)))
}

// --- helpers ---

async fn load_proposal(state: &Arc<AppState>, id: Uuid) -> Result<StoredProposal, ApiError> {
    state
        .proposals
        .get(id)
        .await
        .map_err(|e| {
            error!(error = %e, "proposal lookup failed");
            ApiError::internal("proposal lookup failed")
        })?
        .ok_or_else(|| ApiError::not_found("proposal not found"))
}

fn require_claims(claims: Option<Extension<JwtClaims>>) -> Result<JwtClaims, ApiError> {
    claims
        .map(|Extension(c)| c)
        .ok_or_else(|| ApiError::unauthorized("missing JWT"))
}

fn resolve_seller_id(
    state: &AppState,
    claims: Option<Extension<JwtClaims>>,
) -> Result<(Uuid, Option<String>), ApiError> {
    if state.auth.config().auth_disabled {
        return Ok((Uuid::nil(), None));
    }
    let c = require_claims(claims)?;
    let id = c.current_user_id();
    let doc = if c.document.is_empty() { None } else { Some(c.document.clone()) };
    Ok((id, doc))
}
