//! Order HTTP handlers.

use std::sync::Arc;

use apicash_antifraude::{OnRampDecision, UserScore};
use apicash_auth::{JwtClaims, Role};
use apicash_shared::{AuditEvent, Money, Order, OrderStatus};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::{header, HeaderMap};
use axum::response::Redirect;
use axum::Extension;
use axum::Json;
use chrono::Utc;
use serde::Deserialize;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::dto::{CreateOrderRequest, OrderResponse, RiskScoreRequest};
use crate::error::ApiError;
use crate::state::{AppState, StoredOrder};

/// Raiz do servidor: redireciona para JSON de diagnóstico (evita página em branco no browser).
#[instrument]
pub async fn root() -> Redirect {
    Redirect::temporary("/health")
}

#[instrument]
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "apicash-core"
    }))
}

#[instrument]
pub async fn ready() -> Json<serde_json::Value> {
    let mut development_signals: Vec<&str> = Vec::new();
    if std::env::var("APICASH_ORDERS_PG")
        .map(|v| !matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(true)
    {
        development_signals.push("orders_in_memory");
    }
    Json(serde_json::json!({
        "status": "ready",
        "service": "apicash-core",
        "development_signals": development_signals
    }))
}

#[derive(Debug, Deserialize)]
pub struct InternalSettleRequest {
    pub order_id: Uuid,
}

#[instrument(skip(state, claims, req), fields(buyer_id = %req.buyer_id, seller_id = %req.seller_id))]
pub async fn create_order(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), ApiError> {
    req.validate().map_err(ApiError::bad_request)?;

    // Security rule (critical): bind buyer/seller identities to the authenticated user (JWT `sub`).
    // In production, this must come from JWT. In local dev (`auth_disabled`), we keep a permissive
    // fallback to avoid breaking bootstrap flows.
    let (actor_id, actor_role) = if state.auth.config().auth_disabled {
        (Uuid::nil(), None)
    } else {
        let Some(Extension(c)) = claims else {
            let ts = Utc::now();
            let ev = AuditEvent::UnauthorizedAttempt {
                user_id: None,
                order_id: None,
                action: "OrderCreated".into(),
                reason: "missing JWT".into(),
                timestamp: ts,
            };
            warn!(
                action = "OrderCreated",
                success = false,
                timestamp = %ts,
                audit = ?ev,
                "audit"
            );
            return Err(ApiError::unauthorized("missing JWT"));
        };
        (c.current_user_id(), Some(c.role))
    };

    let (buyer_id, seller_id) = match actor_role {
        None => (req.buyer_id, req.seller_id),
        Some(Role::Buyer) => {
            if req.buyer_id != actor_id {
                let ts = Utc::now();
                let ev = AuditEvent::UnauthorizedAttempt {
                    user_id: Some(actor_id),
                    order_id: None,
                    action: "OrderCreated".into(),
                    reason: "buyer_id mismatch".into(),
                    timestamp: ts,
                };
                warn!(
                    user_id = %actor_id,
                    action = "OrderCreated",
                    success = false,
                    timestamp = %ts,
                    audit = ?ev,
                    "audit"
                );
                return Err(ApiError::unauthorized(
                    "buyer_id must match the authenticated user",
                ));
            }
            (actor_id, req.seller_id)
        }
        Some(Role::Seller) => {
            if req.seller_id != actor_id {
                let ts = Utc::now();
                let ev = AuditEvent::UnauthorizedAttempt {
                    user_id: Some(actor_id),
                    order_id: None,
                    action: "OrderCreated".into(),
                    reason: "seller_id mismatch".into(),
                    timestamp: ts,
                };
                warn!(
                    user_id = %actor_id,
                    action = "OrderCreated",
                    success = false,
                    timestamp = %ts,
                    audit = ?ev,
                    "audit"
                );
                return Err(ApiError::unauthorized(
                    "seller_id must match the authenticated user",
                ));
            }
            (req.buyer_id, actor_id)
        }
        Some(Role::Admin | Role::Platform) => {
            let ts = Utc::now();
            let ev = AuditEvent::UnauthorizedAttempt {
                user_id: Some(actor_id),
                order_id: None,
                action: "OrderCreated".into(),
                reason: "role not allowed to create orders".into(),
                timestamp: ts,
            };
            warn!(
                user_id = %actor_id,
                action = "OrderCreated",
                success = false,
                timestamp = %ts,
                audit = ?ev,
                "audit"
            );
            return Err(ApiError::forbidden(
                "only buyer or seller can create orders",
            ));
        }
    };

    if buyer_id == seller_id {
        let ts = Utc::now();
        let ev = AuditEvent::UnauthorizedAttempt {
            user_id: actor_role.map(|_| actor_id),
            order_id: None,
            action: "OrderCreated".into(),
            reason: "buyer_id equals seller_id".into(),
            timestamp: ts,
        };
        warn!(
            user_id = %actor_id,
            action = "OrderCreated",
            success = false,
            timestamp = %ts,
            audit = ?ev,
            "audit"
        );
        return Err(ApiError::bad_request(
            "buyer_id and seller_id must be different users",
        ));
    }

    let request_user_id = if actor_role.is_some() {
        actor_id
    } else {
        buyer_id
    };

    info!(
        user_id = %request_user_id,
        actor_role = ?actor_role,
        %buyer_id,
        %seller_id,
        action = "OrderCreated",
        "order create requested"
    );

    let amount = Money::from_str_strict(req.amount.trim()).map_err(|e| {
        error!(error = %e, "invalid amount");
        ApiError::bad_request("invalid amount decimal")
    })?;

    let cpf: String = req.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    let order_id = Uuid::new_v4();

    let score = state
        .antifraude
        .calculate_score(buyer_id, &cpf, &req.social_links, Some(amount.into()))
        .await
        .map_err(|e| {
            error!(error = %e, "antifraude failed");
            ApiError::from(e)
        })?;

    if score.decision == OnRampDecision::Block {
        let ts = Utc::now();
        let ev = AuditEvent::OrderCreated {
            user_id: request_user_id,
            order_id,
            buyer_id,
            seller_id,
            success: false,
            timestamp: ts,
        };
        warn!(
            user_id = %request_user_id,
            action = "OrderCreated",
            success = false,
            timestamp = %ts,
            score = score.score,
            audit = ?ev,
            "order rejected by risk policy"
        );
        return Err(ApiError::forbidden(
            "on-ramp blocked by anti-fraud policy for this user",
        ));
    }

    let now = Utc::now();
    let mut order = Order {
        id: order_id,
        buyer_id,
        seller_id,
        amount,
        status: OrderStatus::PendingFunding,
        created_at: now,
        updated_at: now,
    };

    let memo = format!("order:{order_id}");
    let on_ramp = state
        .anchor
        .deposit_pix(order.amount, memo.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "anchor on-ramp (PIX → BRLx) failed");
            ApiError::from(e)
        })?;

    let pix_ok = on_ramp
        .pix_br_code
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !pix_ok {
        return Err(ApiError::bad_gateway(
            "anchor on-ramp did not return pix_br_code; cannot proceed with PIX funding",
        ));
    }

    let funding_instruction =
        Some("Complete o depósito usando o PIX copia-e-cola retornado pelo provedor.".to_string());

    order.status = OrderStatus::PendingFunding;
    order.updated_at = Utc::now();

    let risk_decision = decision_str(score.decision);
    let desc = req
        .description
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let soroban_mode = "pending_funding".to_string();
    let anchor_tx_hash = Some(on_ramp.stellar_tx_hash.clone())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned());
    let stored = StoredOrder {
        order: order.clone(),
        custody_id: None,
        anchor_tx_hash: anchor_tx_hash.clone(),
        fiat_rail: on_ramp.fiat_rail.clone(),
        gateway_in_tx_id: on_ramp.transaction_id.clone(),
        funding_reference: on_ramp.external_id.clone(),
        pix_br_code: on_ramp.pix_br_code.clone(),
        funding_instruction: funding_instruction.clone(),
        risk_score: score.score,
        risk_decision: risk_decision.to_string(),
        description: desc.clone(),
        off_ramp_tx_hash: None,
        brlx_escrow_transfer_tx_hash: None,
        soroban_escrow_contract_id: None,
        soroban_lock_tx_hash: None,
        soroban_mode,
        buyer_name: req.buyer_name.clone(),
    };

    state.orders.save(stored).await.map_err(|e| {
        error!(error = %e, "order persistence failed");
        ApiError::internal("order persistence failed")
    })?;

    info!(
        %order_id,
        on_ramp = %on_ramp.stellar_tx_hash,
        rail = %on_ramp.fiat_rail,
        "order created (pending funding)"
    );
    {
        let ts = Utc::now();
        let ev = AuditEvent::OrderCreated {
            user_id: request_user_id,
            order_id,
            buyer_id,
            seller_id,
            success: true,
            timestamp: ts,
        };
        info!(
            user_id = %request_user_id,
            order_id = %order_id,
            action = "OrderCreated",
            success = true,
            timestamp = %ts,
            audit = ?ev,
            "audit"
        );
    }

    let body = OrderResponse {
        id: order.id,
        buyer_id: order.buyer_id,
        seller_id: order.seller_id,
        amount: order.amount.to_string(),
        status: order.status.to_string(),
        fiat_rail: on_ramp.fiat_rail,
        risk_score: score.score,
        risk_decision: risk_decision.to_string(),
        custody_id: None,
        anchor_tx_hash,
        gateway_in_tx_id: on_ramp.transaction_id,
        funding_reference: on_ramp.external_id,
        pix_br_code: on_ramp.pix_br_code,
        funding_instruction,
        description: desc,
        off_ramp_tx_hash: None,
        brlx_escrow_transfer_tx_hash: None,
        soroban_escrow_contract_id: None,
        soroban_lock_tx_hash: None,
        soroban_mode: Some("pending".to_string()),
        buyer_name: req.buyer_name.clone(),
    };

    Ok((StatusCode::CREATED, Json(body)))
}

#[instrument(skip(state), fields(order_id = %id))]
pub async fn get_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrderResponse>, ApiError> {
    let s = state
        .orders
        .get(id)
        .await
        .map_err(|e| {
            error!(error = %e, "order lookup failed");
            ApiError::internal("order lookup failed")
        })?
        .ok_or_else(|| ApiError::not_found("order not found"))?;
    let o = &s.order;
    Ok(Json(OrderResponse {
        id: o.id,
        buyer_id: o.buyer_id,
        seller_id: o.seller_id,
        amount: o.amount.to_string(),
        status: o.status.to_string(),
        fiat_rail: s.fiat_rail.clone(),
        risk_score: s.risk_score,
        risk_decision: s.risk_decision.clone(),
        custody_id: s.custody_id,
        anchor_tx_hash: s.anchor_tx_hash.clone(),
        gateway_in_tx_id: s.gateway_in_tx_id.clone(),
        funding_reference: s.funding_reference.clone(),
        pix_br_code: s.pix_br_code.clone(),
        funding_instruction: s.funding_instruction.clone(),
        description: s.description.clone(),
        off_ramp_tx_hash: s.off_ramp_tx_hash.clone(),
        brlx_escrow_transfer_tx_hash: s.brlx_escrow_transfer_tx_hash.clone(),
        soroban_escrow_contract_id: s.soroban_escrow_contract_id.clone(),
        soroban_lock_tx_hash: s.soroban_lock_tx_hash.clone(),
        soroban_mode: Some(s.soroban_mode.clone()),
        buyer_name: s.buyer_name.clone(),
    }))
}

/// `GET /orders?role=buyer|seller` — lista pedidos do usuário autenticado.
pub async fn list_orders(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = claims.current_user_id();
    let role_param = params.get("role").map(|s| s.as_str()).unwrap_or("buyer");
    let orders = if role_param == "seller" {
        state.orders.list_by_seller(user_id).await
    } else {
        state.orders.list_by_buyer(user_id).await
    }
    .map_err(|e| {
        error!(error = %e, "list_orders failed");
        ApiError::internal("list orders failed")
    })?;
    let items: Vec<serde_json::Value> = orders
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.order.id,
                "buyer_id": s.order.buyer_id,
                "seller_id": s.order.seller_id,
                "amount": s.order.amount.to_string(),
                "status": s.order.status.to_string(),
                "description": s.description,
                "pix_br_code": s.pix_br_code,
                "created_at": s.order.created_at,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "orders": items, "total": items.len() })))
}

/// `GET /wallet` — saldo do vendedor (soma dos pedidos completados como seller).
pub async fn get_wallet(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = claims.current_user_id();
    let orders = state
        .orders
        .list_by_seller(user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "get_wallet failed");
            ApiError::internal("wallet fetch failed")
        })?;
    let completed_sum: rust_decimal::Decimal = orders
        .iter()
        .filter(|s| matches!(s.order.status, apicash_shared::OrderStatus::Completed))
        .map(|s| s.order.amount.decimal())
        .sum();
    let pending_sum: rust_decimal::Decimal = orders
        .iter()
        .filter(|s| matches!(s.order.status, apicash_shared::OrderStatus::InCustody))
        .map(|s| s.order.amount.decimal())
        .sum();
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "available_balance": completed_sum.to_string(),
        "pending_balance": pending_sum.to_string(),
        "currency": "BRL",
    })))
}

/// `GET /seller/dashboard` — KPIs do seller autenticado.
pub async fn seller_dashboard(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = claims.current_user_id();
    let orders = state
        .orders
        .list_by_seller(user_id)
        .await
        .map_err(|e| {
            error!(error = %e, "seller_dashboard failed");
            ApiError::internal("dashboard fetch failed")
        })?;
    let total = orders.len();
    let completed = orders.iter().filter(|s| matches!(s.order.status, apicash_shared::OrderStatus::Completed)).count();
    let in_custody = orders.iter().filter(|s| matches!(s.order.status, apicash_shared::OrderStatus::InCustody)).count();
    let volume: rust_decimal::Decimal = orders.iter().map(|s| s.order.amount.decimal()).sum();
    let completed_volume: rust_decimal::Decimal = orders
        .iter()
        .filter(|s| matches!(s.order.status, apicash_shared::OrderStatus::Completed))
        .map(|s| s.order.amount.decimal())
        .sum();
    Ok(Json(serde_json::json!({
        "seller_id": user_id,
        "total_orders": total,
        "completed_orders": completed,
        "in_custody_orders": in_custody,
        "total_volume_brl": volume.to_string(),
        "completed_volume_brl": completed_volume.to_string(),
    })))
}

/// Pré-cálculo de score anti-fraude com fatores detalhados (sem criar pedido).
#[instrument(skip(state, req), fields(user_id = %req.user_id))]
pub async fn calculate_risk_score(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RiskScoreRequest>,
) -> Result<Json<UserScore>, ApiError> {
    req.validate().map_err(ApiError::bad_request)?;
    let cpf: String = req.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    let score = state
        .antifraude
        .calculate_score(req.user_id, &cpf, &req.social_links, None)
        .await
        .map_err(|e| {
            error!(error = %e, "antifraude risk score failed");
            ApiError::from(e)
        })?;
    Ok(Json(score))
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| {
            h.strip_prefix("Bearer ")
                .or_else(|| h.strip_prefix("bearer "))
        })
}

fn extract_x_api_key(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
}

/// Internal-only score endpoint used by the WhatsApp Agent.
///
/// Security decision: this handler **does not** accept end-user JWT; it accepts only a service API
/// key (`X-API-Key` or `Authorization: Bearer`) matching `APICASH_API_KEY`.
/// This keeps `POST /risk/score` reserved for authenticated user traffic while still allowing the
/// bot to run antifraud checks during conversational flows.
#[instrument(skip(state, headers, req), fields(user_id = %req.user_id))]
pub async fn calculate_risk_score_internal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RiskScoreRequest>,
) -> Result<Json<UserScore>, ApiError> {
    let expected = std::env::var("APICASH_API_KEY").unwrap_or_default();
    if expected.is_empty() {
        // Misconfiguration: internal route exists, but the key was not set.
        return Err(ApiError::internal("internal API key not configured"));
    }
    let got = extract_x_api_key(&headers).or_else(|| extract_bearer(&headers));
    if got != Some(expected.as_str()) {
        return Err(ApiError::unauthorized(
            "missing or invalid internal API key",
        ));
    }

    // Same validation + scoring as the public endpoint.
    req.validate().map_err(ApiError::bad_request)?;
    let cpf: String = req.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    let score = state
        .antifraude
        .calculate_score(req.user_id, &cpf, &req.social_links, None)
        .await
        .map_err(|e| {
            error!(error = %e, "antifraude risk score failed");
            ApiError::from(e)
        })?;
    Ok(Json(score))
}

#[instrument(skip(state, headers, req), fields(order_id = %req.order_id))]
pub async fn settle_order_internal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<InternalSettleRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let expected = std::env::var("APICASH_API_KEY").unwrap_or_default();
    if expected.is_empty() {
        return Err(ApiError::internal("internal API key not configured"));
    }
    let got = extract_x_api_key(&headers).or_else(|| extract_bearer(&headers));
    if got != Some(expected.as_str()) {
        return Err(ApiError::unauthorized(
            "missing or invalid internal API key",
        ));
    }
    settle_order_by_id(&state, req.order_id).await
}

#[instrument(skip(state), fields(order_id = %id))]
pub async fn settle_order_manual(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    settle_order_by_id(&state, id).await
}

pub(crate) async fn settle_order_by_id(
    state: &Arc<AppState>,
    id: Uuid,
) -> Result<Json<serde_json::Value>, ApiError> {
    let mut stored = state
        .orders
        .get(id)
        .await
        .map_err(|e| {
            error!(error = %e, "order lookup failed");
            ApiError::internal("order lookup failed")
        })?
        .ok_or_else(|| ApiError::not_found("order not found"))?;

    if stored.order.status == OrderStatus::InCustody {
        return Ok(Json(serde_json::json!({
            "order_id": id,
            "status": "already_in_custody"
        })));
    }
    if stored.order.status != OrderStatus::PendingFunding {
        return Err(ApiError::bad_request(
            "order must be pending_funding to settle",
        ));
    }

    let tx_id = stored
        .gateway_in_tx_id
        .clone()
        .ok_or_else(|| ApiError::bad_request("missing gateway_in_tx_id"))?;
    let settlement = state
        .anchor
        .poll_funding_settlement(&tx_id, stored.funding_reference.as_deref())
        .await
        .map_err(ApiError::from)?;
    if !settlement.settled {
        return Ok(Json(serde_json::json!({
            "order_id": id,
            "status": "pending_settlement",
            "rail_status": settlement.status
        })));
    }

    let escrow_addr = resolve_escrow_contract_id().map_err(|e| ApiError::bad_request(e))?;
    let memo = stored
        .funding_reference
        .clone()
        .unwrap_or_else(|| format!("order:{id}"));
    let brlx_to_escrow = state
        .anchor
        .transfer_brlx_to_escrow(&escrow_addr, stored.order.amount, &memo)
        .await
        .map_err(ApiError::from)?;
    if apicash_shared::require_testnet() && brlx_to_escrow.is_mock {
        return Err(ApiError::internal(
            "BRLx transfer must be a real Stellar testnet transaction (APICASH_REQUIRE_TESTNET=1)",
        ));
    }
    let custody = state
        .custody
        .lock_funds(&stored.order)
        .await
        .map_err(ApiError::from)?;

    stored.order.status = OrderStatus::InCustody;
    stored.order.updated_at = Utc::now();
    stored.custody_id = Some(custody.id);
    // Preserve Stellar correlation hash from deposit when present; else anchor poll id.
    stored.anchor_tx_hash = stored
        .anchor_tx_hash
        .clone()
        .or_else(|| settlement.transaction_id.clone());
    stored.brlx_escrow_transfer_tx_hash = Some(brlx_to_escrow.tx_hash);
    stored.soroban_escrow_contract_id = custody.soroban_escrow_contract_id.clone();
    stored.soroban_lock_tx_hash = custody.soroban_lock_tx_hash.clone();
    stored.soroban_mode = if custody.soroban_is_mock {
        "mock".into()
    } else {
        "soroban".into()
    };

    state.orders.update(stored).await.map_err(|e| {
        error!(error = %e, "order settlement persistence failed");
        ApiError::internal("order settlement persistence failed")
    })?;

    Ok(Json(serde_json::json!({
        "order_id": id,
        "status": "in_custody"
    })))
}

/// Off-ramp mock: BRLx → PIX após custódia liberada (`order` em `completed`).
#[instrument(skip(state), fields(order_id = %id))]
pub async fn order_off_ramp(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<OffRampBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key = body
        .destination_pix_key
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "mock+offramp@apicash.dev".into());

    let mut stored = state
        .orders
        .get(id)
        .await
        .map_err(|e| {
            error!(error = %e, "order lookup failed");
            ApiError::internal("order lookup failed")
        })?
        .ok_or_else(|| ApiError::not_found("order not found"))?;

    if let Some(ref h) = stored.off_ramp_tx_hash {
        info!(%h, "off-ramp already executed (idempotent)");
        return Ok(Json(serde_json::json!({
            "order_id": id,
            "tx_hash": h,
            "status": "already_completed",
            "destination_pix_key": key,
        })));
    }

    if stored.order.status != OrderStatus::Completed {
        return Err(ApiError::bad_request(
            "order must be completed (custody released) before off-ramp",
        ));
    }

    let resp = state
        .anchor
        .withdraw_to_pix(
            stored.order.amount,
            key.clone(),
            format!("order:{id}:offramp:v1"),
            format!("offramp order:{id}"),
        )
        .await
        .map_err(|e| {
            error!(error = %e, "anchor off-ramp failed");
            ApiError::from(e)
        })?;

    stored.off_ramp_tx_hash = Some(resp.tx_hash.clone());
    state.orders.update(stored).await.map_err(|e| {
        error!(error = %e, "order off-ramp persistence failed");
        ApiError::internal("order off-ramp persistence failed")
    })?;

    info!(tx = %resp.tx_hash, %id, "off-ramp completed");

    Ok(Json(serde_json::json!({
        "order_id": id,
        "tx_hash": resp.tx_hash,
        "status": resp.status,
        "received_pix": resp.received_pix,
        "destination_pix_key": key,
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct OffRampBody {
    #[serde(default)]
    pub destination_pix_key: Option<String>,
}

fn decision_str(d: OnRampDecision) -> &'static str {
    match d {
        OnRampDecision::Approve => "approve",
        OnRampDecision::Review => "review",
        OnRampDecision::Block => "block",
    }
}

fn resolve_escrow_contract_id() -> Result<String, &'static str> {
    match std::env::var("APICASH_SOROBAN_ESCROW_CONTRACT_ID") {
        Ok(v) if !v.trim().is_empty() && !v.to_ascii_lowercase().contains("mock") => Ok(v),
        _ if apicash_shared::require_testnet() => Err(
            "APICASH_SOROBAN_ESCROW_CONTRACT_ID required for testnet (run scripts/soroban-testnet-deploy.sh)",
        ),
        _ => Ok("mock_escrow_contract".into()),
    }
}

/// Core order creation logic shared between `POST /orders` and `POST /proposals/{id}/accept`.
///
/// Skips JWT binding (caller must verify identity before invoking). Performs anti-fraud scoring,
/// PIX on-ramp, and persistence; returns the full order response on success.
#[instrument(skip(state, social_links, description), fields(%buyer_id, %seller_id))]
pub(crate) async fn create_escrow_order_core(
    state: &Arc<AppState>,
    buyer_id: Uuid,
    seller_id: Uuid,
    amount_str: &str,
    cpf: &str,
    social_links: &[String],
    description: Option<&str>,
) -> Result<crate::dto::OrderResponse, ApiError> {
    let amount = Money::from_str_strict(amount_str.trim()).map_err(|e| {
        error!(error = %e, "invalid amount");
        ApiError::bad_request("invalid amount decimal")
    })?;

    let cpf_digits: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    let order_id = Uuid::new_v4();

    let score = state
        .antifraude
        .calculate_score(buyer_id, &cpf_digits, social_links, Some(amount.into()))
        .await
        .map_err(|e| {
            error!(error = %e, "antifraude failed");
            ApiError::from(e)
        })?;

    if score.decision == OnRampDecision::Block {
        warn!(
            %buyer_id,
            %order_id,
            score = score.score,
            "order creation blocked by anti-fraud policy"
        );
        return Err(ApiError::forbidden(
            "on-ramp blocked by anti-fraud policy for this user",
        ));
    }

    let now = Utc::now();
    let mut order = Order {
        id: order_id,
        buyer_id,
        seller_id,
        amount,
        status: OrderStatus::PendingFunding,
        created_at: now,
        updated_at: now,
    };

    let memo = format!("order:{order_id}");
    let on_ramp = state
        .anchor
        .deposit_pix(order.amount, memo.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "anchor on-ramp (PIX → BRLx) failed");
            ApiError::from(e)
        })?;

    let pix_ok = on_ramp
        .pix_br_code
        .as_ref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if !pix_ok {
        return Err(ApiError::bad_gateway(
            "anchor on-ramp did not return pix_br_code; cannot proceed with PIX funding",
        ));
    }

    let funding_instruction =
        Some("Complete o depósito usando o PIX copia-e-cola retornado pelo provedor.".to_string());

    order.status = OrderStatus::PendingFunding;
    order.updated_at = Utc::now();

    let risk_decision = decision_str(score.decision);
    let desc = description
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let soroban_mode = "pending_funding".to_string();
    let anchor_tx_hash = Some(on_ramp.stellar_tx_hash.clone())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_owned());
    let stored = StoredOrder {
        order: order.clone(),
        custody_id: None,
        anchor_tx_hash: anchor_tx_hash.clone(),
        fiat_rail: on_ramp.fiat_rail.clone(),
        gateway_in_tx_id: on_ramp.transaction_id.clone(),
        funding_reference: on_ramp.external_id.clone(),
        pix_br_code: on_ramp.pix_br_code.clone(),
        funding_instruction: funding_instruction.clone(),
        risk_score: score.score,
        risk_decision: risk_decision.to_string(),
        description: desc.clone(),
        off_ramp_tx_hash: None,
        brlx_escrow_transfer_tx_hash: None,
        soroban_escrow_contract_id: None,
        soroban_lock_tx_hash: None,
        soroban_mode,
        buyer_name: None,
    };

    state.orders.save(stored).await.map_err(|e| {
        error!(error = %e, "order persistence failed");
        ApiError::internal("order persistence failed")
    })?;

    info!(%order_id, rail = %on_ramp.fiat_rail, "order created via core helper (pending funding)");

    Ok(crate::dto::OrderResponse {
        id: order.id,
        buyer_id: order.buyer_id,
        seller_id: order.seller_id,
        amount: order.amount.to_string(),
        status: order.status.to_string(),
        fiat_rail: on_ramp.fiat_rail,
        risk_score: score.score,
        risk_decision: risk_decision.to_string(),
        custody_id: None,
        anchor_tx_hash,
        gateway_in_tx_id: on_ramp.transaction_id,
        funding_reference: on_ramp.external_id,
        pix_br_code: on_ramp.pix_br_code,
        funding_instruction,
        description: desc,
        off_ramp_tx_hash: None,
        brlx_escrow_transfer_tx_hash: None,
        soroban_escrow_contract_id: None,
        soroban_lock_tx_hash: None,
        soroban_mode: Some("pending".to_string()),
        buyer_name: None,
    })
}

/// Open a dispute for an existing order.
///
/// Either the buyer or seller of the order may open a dispute. The order must be in
/// `pending_funding` or `in_custody` state.
#[instrument(skip(state, claims, body), fields(order_id = %id))]
pub async fn open_dispute(
    State(state): State<Arc<AppState>>,
    claims: Option<Extension<JwtClaims>>,
    Path(id): Path<Uuid>,
    Json(body): Json<DisputeBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stored = state
        .orders
        .get(id)
        .await
        .map_err(|e| {
            error!(error = %e, "order lookup failed");
            ApiError::internal("order lookup failed")
        })?
        .ok_or_else(|| ApiError::not_found("order not found"))?;

    let actor_id = if state.auth.config().auth_disabled {
        stored.order.buyer_id
    } else {
        let Some(Extension(c)) = claims else {
            return Err(ApiError::unauthorized("missing JWT"));
        };
        c.current_user_id()
    };

    let is_party = actor_id == stored.order.buyer_id || actor_id == stored.order.seller_id;
    if !is_party {
        return Err(ApiError::forbidden(
            "only the buyer or seller of this order can open a dispute",
        ));
    }

    let allowed = matches!(
        stored.order.status,
        OrderStatus::PendingFunding | OrderStatus::InCustody
    );
    if !allowed {
        return Err(ApiError::bad_request(
            "disputes can only be opened on orders in pending_funding or in_custody state",
        ));
    }

    let dispute_id = Uuid::new_v4();
    let opened_by = if actor_id == stored.order.buyer_id {
        "buyer"
    } else {
        "seller"
    };

    info!(
        %dispute_id,
        %id,
        %opened_by,
        "dispute opened"
    );

    Ok(Json(serde_json::json!({
        "dispute_id": dispute_id,
        "order_id": id,
        "status": "open",
        "opened_by": opened_by,
        "reason": body.reason,
        "message": "Disputa registrada. O suporte responde em até 1 dia útil."
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct DisputeBody {
    #[serde(default)]
    pub reason: Option<String>,
}
