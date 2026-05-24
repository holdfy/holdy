//! Composed Axum router: tracing, optional auth, REST routes.

use std::sync::Arc;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;

use crate::handlers::{
    auth_handler, custody_handler, order_handler, payment_handler, proposal_handler,
    testnet_handler, webhook_handler,
};
use crate::middleware::{auth_middleware, build_x402_layer};
use crate::state::AppState;

/// Builds the HTTP application with shared [`AppState`].
pub fn create_router(state: Arc<AppState>) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(12)
            .finish()
            .expect("governor config"),
    );

    let auth_routes = Router::new()
        .route("/login", post(auth_handler::login))
        .route("/refresh", post(auth_handler::refresh))
        .layer(GovernorLayer::new(governor_conf.clone()))
        .with_state(state.clone());

    let mut protected = Router::new()
        .route("/orders", get(order_handler::list_orders).post(order_handler::create_order))
        .route("/orders/{id}", get(order_handler::get_order))
        .route("/wallet", get(order_handler::get_wallet))
        .route("/seller/dashboard", get(order_handler::seller_dashboard))
        .route(
            "/orders/{id}/settle",
            post(order_handler::settle_order_manual),
        )
        .route("/orders/{id}/off-ramp", post(order_handler::order_off_ramp))
        .route("/orders/{id}/dispute", post(order_handler::open_dispute))
        .route("/risk/score", post(order_handler::calculate_risk_score))
        .route("/payments/pix", post(payment_handler::create_pix_payment))
        .route("/custody/release", post(custody_handler::release_custody))
        // Proposal flow: two-party escrow negotiation
        .route("/proposals", post(proposal_handler::create_proposal))
        .route("/proposals/{id}", get(proposal_handler::get_proposal))
        .route(
            "/proposals/{id}/accept",
            post(proposal_handler::accept_proposal),
        )
        .route(
            "/proposals/{id}/reject",
            post(proposal_handler::reject_proposal),
        );

    if let Some(x402) = build_x402_layer(state.clone()) {
        protected = protected.layer(x402);
    }

    let protected = protected.layer(middleware::from_fn_with_state(
        state.clone(),
        auth_middleware,
    ));

    // Internal service-to-service routes.
    //
    // Security decision: keep the public `POST /risk/score` bound to end-user JWT (via middleware),
    // but allow the WhatsApp Agent to call a dedicated internal endpoint using a service API key
    // (`X-API-Key` = `APICASH_API_KEY`). This avoids requiring a full end-user auth dance before
    // the bot can pre-calculate score, while keeping the public surface locked down.
    let internal = Router::new()
        .route(
            "/internal/risk/score",
            post(order_handler::calculate_risk_score_internal),
        )
        .route(
            "/internal/orders/settle",
            post(order_handler::settle_order_internal),
        )
        .layer(GovernorLayer::new(governor_conf.clone()))
        .with_state(state.clone());

    // Webhook route: autenticação própria via HMAC-SHA256 (X-Webhook-Signature),
    // não usa X-API-Key nem JWT. Sem rate limit (Gatebox controla a frequência).
    let webhooks = Router::new()
        .route(
            "/internal/webhook/pix",
            post(webhook_handler::receive_pix_webhook),
        )
        .with_state(state.clone());

    Router::new()
        .route("/", get(order_handler::root))
        .route("/health", get(order_handler::health))
        .route("/ready", get(order_handler::ready))
        .route(
            "/testnet/transactions",
            get(testnet_handler::recent_testnet_transactions),
        )
        .route("/openapi.json", get(openapi_spec_protected))
        .route("/docs", get(swagger_ui))
        .route("/docs/flows", get(flows_page))
        .nest("/auth", auth_routes)
        .merge(internal)
        .merge(protected)
        .merge(webhooks)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Serves the OpenAPI 3.0 specification for the APICash public API.
async fn openapi_spec() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
      "openapi": "3.0.3",
      "info": {
        "title": "APICash API",
        "description": "Escrow PIX ↔ Stellar BRLx. Mesmas funcionalidades do agente WhatsApp expostas como REST.",
        "version": "0.1.0",
        "contact": { "name": "HoldFy", "url": "https://holdfy.com" }
      },
      "servers": [{ "url": "/", "description": "Este servidor" }],
      "tags": [
        { "name": "auth", "description": "Autenticação JWT" },
        { "name": "proposals", "description": "Fluxo de proposta (2 partes): seller cria, buyer aceita" },
        { "name": "orders", "description": "Pedidos de escrow" },
        { "name": "custody", "description": "Custódia e liberação de fundos" },
        { "name": "payments", "description": "Pagamentos PIX diretos" }
      ],
      "components": {
        "securitySchemes": {
          "bearerAuth": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT" }
        },
        "schemas": {
          "Error": {
            "type": "object",
            "properties": { "error": { "type": "string" } }
          },
          "CreateProposalRequest": {
            "type": "object", "required": ["buyer_id", "amount"],
            "properties": {
              "buyer_id": { "type": "string", "format": "uuid" },
              "amount": { "type": "string", "example": "100.50", "description": "Valor em BRL (decimal)" },
              "description": { "type": "string", "example": "Consultoria de software" }
            }
          },
          "ProposalResponse": {
            "type": "object",
            "properties": {
              "id": { "type": "string", "format": "uuid" },
              "seller_id": { "type": "string", "format": "uuid" },
              "buyer_id": { "type": "string", "format": "uuid" },
              "amount": { "type": "string" },
              "description": { "type": "string" },
              "status": { "type": "string", "enum": ["pending","accepted","rejected","expired"] },
              "created_at": { "type": "string", "format": "date-time" },
              "expires_at": { "type": "string", "format": "date-time" },
              "order_id": { "type": "string", "format": "uuid" }
            }
          },
          "AcceptProposalResponse": {
            "type": "object",
            "properties": {
              "proposal_id": { "type": "string", "format": "uuid" },
              "order_id": { "type": "string", "format": "uuid" },
              "pix_br_code": { "type": "string", "description": "PIX copia-e-cola EMV (pagar no app do banco)" },
              "amount": { "type": "string" },
              "status": { "type": "string", "example": "accepted" },
              "funding_instruction": { "type": "string" }
            }
          },
          "OrderResponse": {
            "type": "object",
            "properties": {
              "id": { "type": "string", "format": "uuid" },
              "buyer_id": { "type": "string", "format": "uuid" },
              "seller_id": { "type": "string", "format": "uuid" },
              "amount": { "type": "string" },
              "status": { "type": "string", "enum": ["pending_funding","in_custody","completed","cancelled","failed"] },
              "pix_br_code": { "type": "string" },
              "fiat_rail": { "type": "string" },
              "risk_score": { "type": "integer" },
              "risk_decision": { "type": "string", "enum": ["approve","review","block"] }
            }
          }
        }
      },
      "paths": {
        "/auth/login": {
          "post": {
            "tags": ["auth"], "summary": "Login (obter JWT)",
            "requestBody": {
              "required": true,
              "content": { "application/json": { "schema": { "type": "object", "required": ["username","password"],
                "properties": { "username": { "type": "string" }, "password": { "type": "string" } } } } }
            },
            "responses": {
              "200": { "description": "JWT access + refresh token" },
              "401": { "description": "Credenciais inválidas" }
            }
          }
        },
        "/proposals": {
          "post": {
            "tags": ["proposals"], "summary": "Seller cria uma proposta para buyer",
            "security": [{ "bearerAuth": [] }],
            "requestBody": {
              "required": true,
              "content": { "application/json": { "schema": { "$ref": "#/components/schemas/CreateProposalRequest" } } }
            },
            "responses": {
              "201": { "description": "Proposta criada", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ProposalResponse" } } } },
              "400": { "$ref": "#/components/responses/BadRequest" },
              "401": { "$ref": "#/components/responses/Unauthorized" }
            }
          }
        },
        "/proposals/{id}": {
          "get": {
            "tags": ["proposals"], "summary": "Consultar status da proposta",
            "security": [{ "bearerAuth": [] }],
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
            "responses": {
              "200": { "description": "Proposta", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ProposalResponse" } } } },
              "404": { "$ref": "#/components/responses/NotFound" }
            }
          }
        },
        "/proposals/{id}/accept": {
          "post": {
            "tags": ["proposals"], "summary": "Buyer aceita → cria pedido + retorna PIX QR",
            "security": [{ "bearerAuth": [] }],
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
            "requestBody": {
              "content": { "application/json": { "schema": { "type": "object",
                "properties": { "cpf": { "type": "string", "description": "CPF do buyer (11 dígitos)" },
                  "social_links": { "type": "array", "items": { "type": "string" } } } } } }
            },
            "responses": {
              "200": { "description": "Pedido criado + PIX copia-e-cola", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/AcceptProposalResponse" } } } },
              "400": { "$ref": "#/components/responses/BadRequest" },
              "403": { "description": "Antifraude bloqueou ou actor não é o buyer" }
            }
          }
        },
        "/proposals/{id}/reject": {
          "post": {
            "tags": ["proposals"], "summary": "Buyer rejeita a proposta",
            "security": [{ "bearerAuth": [] }],
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
            "responses": {
              "200": { "description": "Proposta rejeitada", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ProposalResponse" } } } }
            }
          }
        },
        "/orders": {
          "post": {
            "tags": ["orders"], "summary": "Criar pedido de escrow diretamente (sem proposta)",
            "security": [{ "bearerAuth": [] }],
            "requestBody": {
              "required": true,
              "content": { "application/json": { "schema": { "type": "object", "required": ["buyer_id","seller_id","amount","cpf"],
                "properties": {
                  "buyer_id": { "type": "string", "format": "uuid" },
                  "seller_id": { "type": "string", "format": "uuid" },
                  "amount": { "type": "string", "example": "100.50" },
                  "cpf": { "type": "string", "example": "12345678901" },
                  "social_links": { "type": "array", "items": { "type": "string" } },
                  "description": { "type": "string" }
                } } } }
            },
            "responses": {
              "201": { "description": "Pedido criado", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/OrderResponse" } } } }
            }
          }
        },
        "/orders/{id}": {
          "get": {
            "tags": ["orders"], "summary": "Consultar pedido",
            "security": [{ "bearerAuth": [] }],
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
            "responses": {
              "200": { "description": "Pedido", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/OrderResponse" } } } },
              "404": { "$ref": "#/components/responses/NotFound" }
            }
          }
        },
        "/orders/{id}/dispute": {
          "post": {
            "tags": ["orders"], "summary": "Abrir disputa (buyer ou seller)",
            "security": [{ "bearerAuth": [] }],
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
            "requestBody": {
              "content": { "application/json": { "schema": { "type": "object",
                "properties": { "reason": { "type": "string" } } } } }
            },
            "responses": {
              "200": { "description": "Disputa aberta", "content": { "application/json": { "schema": { "type": "object",
                "properties": {
                  "dispute_id": { "type": "string", "format": "uuid" },
                  "order_id": { "type": "string", "format": "uuid" },
                  "status": { "type": "string", "example": "open" },
                  "opened_by": { "type": "string", "enum": ["buyer","seller"] },
                  "message": { "type": "string" }
                } } } } }
            }
          }
        },
        "/custody/release": {
          "post": {
            "tags": ["custody"], "summary": "Buyer confirma entrega → libera escrow para seller",
            "security": [{ "bearerAuth": [] }],
            "requestBody": {
              "required": true,
              "content": { "application/json": { "schema": { "type": "object", "required": ["order_id","released_by","idempotency_key"],
                "properties": {
                  "order_id": { "type": "string", "format": "uuid" },
                  "released_by": { "type": "string", "format": "uuid" },
                  "idempotency_key": { "type": "string" }
                } } } }
            },
            "responses": {
              "200": { "description": "Custódia liberada" }
            }
          }
        },
        "/orders/{id}/off-ramp": {
          "post": {
            "tags": ["orders"], "summary": "Seller recebe BRLx via PIX (off-ramp)",
            "security": [{ "bearerAuth": [] }],
            "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }],
            "requestBody": {
              "content": { "application/json": { "schema": { "type": "object",
                "properties": { "destination_pix_key": { "type": "string" } } } } }
            },
            "responses": {
              "200": { "description": "Off-ramp executado" }
            }
          }
        }
      },
      "components": {
        "responses": {
          "BadRequest": { "description": "Requisição inválida", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } },
          "Unauthorized": { "description": "JWT ausente ou inválido", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } },
          "NotFound": { "description": "Recurso não encontrado", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } }
        },
        "securitySchemes": {
          "bearerAuth": { "type": "http", "scheme": "bearer", "bearerFormat": "JWT" }
        },
        "schemas": {
          "Error": { "type": "object", "properties": { "error": { "type": "string" } } },
          "ProposalResponse": {
            "type": "object",
            "properties": {
              "id": { "type": "string", "format": "uuid" },
              "seller_id": { "type": "string", "format": "uuid" },
              "buyer_id": { "type": "string", "format": "uuid" },
              "amount": { "type": "string" },
              "description": { "type": "string" },
              "status": { "type": "string", "enum": ["pending","accepted","rejected","expired"] },
              "created_at": { "type": "string", "format": "date-time" },
              "expires_at": { "type": "string", "format": "date-time" },
              "order_id": { "type": "string", "format": "uuid" }
            }
          },
          "AcceptProposalResponse": {
            "type": "object",
            "properties": {
              "proposal_id": { "type": "string", "format": "uuid" },
              "order_id": { "type": "string", "format": "uuid" },
              "pix_br_code": { "type": "string" },
              "amount": { "type": "string" },
              "status": { "type": "string" },
              "funding_instruction": { "type": "string" }
            }
          },
          "OrderResponse": {
            "type": "object",
            "properties": {
              "id": { "type": "string", "format": "uuid" },
              "buyer_id": { "type": "string", "format": "uuid" },
              "seller_id": { "type": "string", "format": "uuid" },
              "amount": { "type": "string" },
              "status": { "type": "string" },
              "pix_br_code": { "type": "string" },
              "fiat_rail": { "type": "string" },
              "risk_score": { "type": "integer" },
              "risk_decision": { "type": "string" }
            }
          },
          "CreateProposalRequest": {
            "type": "object", "required": ["buyer_id","amount"],
            "properties": {
              "buyer_id": { "type": "string", "format": "uuid" },
              "amount": { "type": "string", "example": "100.50" },
              "description": { "type": "string" }
            }
          }
        }
      }
    }))
}

/// Verifica Basic Auth para as rotas de documentação.
///
/// Credenciais: username `holdfy` + `APICASH_DOCS_PASSWORD` (default `stellar@37graus`).
fn docs_auth_ok(headers: &axum::http::HeaderMap) -> bool {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;

    let password = std::env::var("APICASH_DOCS_PASSWORD")
        .unwrap_or_else(|_| "stellar@37graus".to_string());
    let username = std::env::var("APICASH_DOCS_USERNAME")
        .unwrap_or_else(|_| "holdfy".to_string());

    let expected_b64 = STANDARD.encode(format!("{username}:{password}"));

    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Basic "))
        .map(|got| got.trim() == expected_b64)
        .unwrap_or(false)
}

type DocsResponse = axum::response::Response;

fn unauthorized_docs() -> DocsResponse {
    use axum::response::IntoResponse;
    (
        axum::http::StatusCode::UNAUTHORIZED,
        [(axum::http::header::WWW_AUTHENTICATE, "Basic realm=\"APICash Docs\"")],
        axum::Json(serde_json::json!({ "error": "authentication required" })),
    )
        .into_response()
}

/// GET /openapi.json — OpenAPI 3.0 spec (requer Basic Auth).
async fn openapi_spec_protected(
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !docs_auth_ok(&headers) {
        return unauthorized_docs();
    }
    openapi_spec().await.into_response()
}

/// GET /docs — Swagger UI embarcado (requer Basic Auth).
async fn swagger_ui(headers: axum::http::HeaderMap) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !docs_auth_ok(&headers) {
        return unauthorized_docs();
    }
    axum::response::Html(SWAGGER_UI_HTML).into_response()
}

/// GET /docs/flows — BPM diagrams (Mermaid.js, requer Basic Auth).
async fn flows_page(headers: axum::http::HeaderMap) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !docs_auth_ok(&headers) {
        return unauthorized_docs();
    }
    axum::response::Html(FLOWS_HTML).into_response()
}

const FLOWS_HTML: &str = r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>APICash — BPM Flows</title>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"></script>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    body { background: #0f172a; color: #e2e8f0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
    header { background: #1e293b; padding: 1.25rem 2rem; border-bottom: 1px solid #334155; display: flex; align-items: center; gap: 1rem; }
    header h1 { font-size: 1.25rem; font-weight: 700; color: #f8fafc; }
    header span { font-size: 0.8rem; color: #94a3b8; background: #0f172a; padding: 0.2rem 0.6rem; border-radius: 999px; }
    nav { display: flex; gap: 0.5rem; padding: 1rem 2rem; background: #1e293b; border-bottom: 1px solid #334155; overflow-x: auto; }
    nav button { background: #334155; color: #cbd5e1; border: none; border-radius: 0.5rem; padding: 0.5rem 1.1rem; cursor: pointer; font-size: 0.875rem; white-space: nowrap; transition: background 0.15s, color 0.15s; }
    nav button:hover { background: #475569; color: #f1f5f9; }
    nav button.active { background: #6366f1; color: #fff; }
    main { padding: 2rem; max-width: 1400px; }
    .flow { display: none; }
    .flow.visible { display: block; }
    .flow h2 { font-size: 1.1rem; font-weight: 600; margin-bottom: 0.5rem; color: #818cf8; }
    .flow p.desc { color: #94a3b8; font-size: 0.875rem; margin-bottom: 1.5rem; line-height: 1.6; }
    .diagram-wrap { background: #1e293b; border-radius: 0.75rem; padding: 2rem; overflow-x: auto; border: 1px solid #334155; }
    .mermaid { min-width: 600px; }
  </style>
</head>
<body>
  <header>
    <h1>APICash — BPM Flows</h1>
    <span>Diagramas de processo</span>
  </header>
  <nav id="tabs">
    <button class="active" onclick="show(0, this)">1. REST API</button>
    <button onclick="show(1, this)">2. WhatsApp Bot</button>
    <button onclick="show(2, this)">3. PIX Webhook</button>
    <button onclick="show(3, this)">4. Soroban/Stellar</button>
  </nav>
  <main>

    <div class="flow visible" id="flow0">
      <h2>Fluxo 1 — REST API: Proposta → Escrow → Liberação</h2>
      <p class="desc">Fluxo completo do produto via REST. Seller cria uma proposta, buyer aceita e paga via PIX, fundos são travados no escrow Soroban, buyer confirma entrega e seller recebe via off-ramp PIX.</p>
      <div class="diagram-wrap">
        <div class="mermaid">
sequenceDiagram
    autonumber
    actor Seller
    actor Buyer
    participant Core as APICash Core
    participant Anti as Antifraude
    participant Gate as Gatebox/Anchor
    participant Stellar as Stellar Network

    Seller->>Core: POST /auth/login
    Core-->>Seller: JWT (role=seller)
    Seller->>Core: POST /proposals { buyer_id, amount }
    Core-->>Seller: 201 { proposal_id, expires_at }
    Note over Buyer,Core: Buyer recebe proposal_id por canal externo

    Buyer->>Core: POST /auth/login
    Core-->>Buyer: JWT (role=buyer)
    Buyer->>Core: POST /proposals/{id}/accept { cpf? }
    Core->>Anti: calculate_score(cpf, social_links)
    Anti-->>Core: { score, decision }

    alt decision == BLOCK
        Core-->>Buyer: 403 Antifraude bloqueou
    else APPROVE / REVIEW
        Core->>Gate: POST /api/v1/pix/qrcode { amount, ttl=30min }
        Gate-->>Core: { qr_code, transaction_id }
        Core->>Core: Salva order (pending_funding)
        Core-->>Buyer: 200 { order_id, pix_br_code }
    end

    Note over Buyer,Gate: Buyer paga PIX no app do banco

    Gate->>Core: POST /internal/webhook/pix X-Webhook-Signature
    Core->>Core: Verifica HMAC-SHA256
    Core->>Stellar: BRLx transfer issuer→buyer
    Core->>Stellar: BRLx transfer buyer→escrow
    Core->>Stellar: ESCROW lock(order_key)
    Core->>Core: status → in_custody
    Core-->>Gate: 200 OK

    Note over Buyer,Core: Buyer recebe produto/serviço

    Buyer->>Core: POST /custody/release { order_id }
    Core->>Stellar: ESCROW release(order_key)
    Note right of Stellar: yield split: buyer 70% seller 10% platform 20%
    Core->>Core: status → completed
    Core-->>Buyer: 200 { status: completed }

    Seller->>Core: POST /orders/{id}/off-ramp { pix_key }
    Core->>Gate: PIX OUT → seller
        </div>
      </div>
    </div>

    <div class="flow" id="flow1">
      <h2>Fluxo 2 — WhatsApp Bot: Conversa Seller + Buyer</h2>
      <p class="desc">Agente conversacional multi-device. Seller inicia uma venda conversando com o bot; buyer recebe proposta e aceita ou rejeita. Antifraude é verificado antes de gerar o QR PIX.</p>
      <div class="diagram-wrap">
        <div class="mermaid">
sequenceDiagram
    autonumber
    actor Seller as Seller (WhatsApp)
    actor Buyer as Buyer (WhatsApp)
    participant Bot as APICash Bot
    participant Anti as Antifraude

    Seller->>Bot: menciona vender
    Bot-->>Seller: Qual o valor da venda?
    Seller->>Bot: envia valor ex 100.00
    Bot-->>Seller: Qual o telefone do comprador?
    Seller->>Bot: envia telefone
    Bot->>Bot: cria proposta TTL 60min
    Bot-->>Seller: Proposta enviada aguardando buyer

    Bot-->>Buyer: Nova proposta do seller - aceita?

    alt Buyer rejeita
        Buyer->>Bot: NAO
        Bot-->>Buyer: Proposta recusada
        Bot-->>Seller: Buyer recusou
    else Buyer aceita
        Buyer->>Bot: SIM
        Bot->>Anti: calculate_score(buyer)
        Anti-->>Bot: score + decision

        alt BLOCK
            Bot-->>Buyer: Operacao nao permitida
        else APPROVE ou REVIEW
            Bot-->>Buyer: PIX copia-e-cola
            Note over Buyer,Bot: Buyer paga PIX no app do banco
            Note over Bot: webhook confirma pagamento
            Bot-->>Buyer: Fundos em custodia - voce recebeu?
            Bot-->>Seller: Aguardando confirmacao do buyer

            alt Buyer confirma entrega
                Buyer->>Bot: confirma recebido
                Bot->>Bot: release escrow
                Bot-->>Buyer: Obrigado pela confirmacao
                Bot-->>Seller: Pagamento liberado
            else Buyer abre disputa
                Buyer->>Bot: abre disputa com motivo
                Bot-->>Buyer: Disputa registrada equipe notificada
                Bot-->>Seller: Disputa aberta pelo buyer
            end
        end
    end
        </div>
      </div>
    </div>

    <div class="flow" id="flow2">
      <h2>Fluxo 3 — PIX: Geração de QR e Confirmação via Webhook</h2>
      <p class="desc">Detalha o canal de pagamento PIX: Gatebox gera o BR Code EMV via Anchor, buyer paga, Anchor confirma e Gatebox dispara o webhook HMAC para o APICash Core processar o settlement.</p>
      <div class="diagram-wrap">
        <div class="mermaid">
sequenceDiagram
    autonumber
    participant Core as APICash Core
    participant Gate as Gatebox
    participant Anchor as Anchor PSP
    actor Buyer as Buyer (banco)

    rect rgb(20,40,70)
        Note over Core,Anchor: Subfluxo A — Geração do QR Code
        Core->>Gate: POST /api/v1/pix/qrcode { amount, pix_key, ttl=1800s }
        Gate->>Anchor: Solicita EMV BR Code
        Anchor-->>Gate: BR Code gerado
        Gate-->>Core: { qr_code, transaction_id }
        Core->>Core: Salva gateway_in_tx_id + pix_br_code
    end

    Note over Buyer,Anchor: Buyer paga PIX no app do banco

    rect rgb(20,60,40)
        Note over Core,Anchor: Subfluxo B — Confirmação via Webhook
        Buyer->>Anchor: Realiza pagamento PIX
        Anchor->>Gate: Pagamento confirmado
        Gate->>Core: POST /internal/webhook/pix X-Webhook-Signature: hex(HMAC-SHA256)

        Core->>Core: Verifica HMAC-SHA256
        alt Signature inválida
            Core-->>Gate: 401 Unauthorized
        else pix_out / reversal / test
            Core-->>Gate: 200 OK ignorado
        else status confirmado
            Core->>Core: Busca order por gateway_in_tx_id
            Core->>Core: settle_order_by_id()
            Core-->>Gate: 200 OK settled:true
        end
    end

    Note over Core: Poller como fallback se webhook falhar
        </div>
      </div>
    </div>

    <div class="flow" id="flow3">
      <h2>Fluxo 4 — Soroban/Stellar: Operações On-Chain</h2>
      <p class="desc">Detalha as três operações blockchain: emissão de BRLx pelo issuer, transferência ao escrow Soroban, lock do contrato e release com distribuição de yield (buyer 70% / seller 10% / platform 20%).</p>
      <div class="diagram-wrap">
        <div class="mermaid">
sequenceDiagram
    autonumber
    participant Anchor as APICash Anchor
    participant Custody as APICash Custody
    participant CLI as stellar CLI
    participant RPC as Soroban RPC

    rect rgb(40,20,70)
        Note over Anchor,RPC: Subfluxo A — Emissão BRLx + Transferência ao Escrow
        Anchor->>Anchor: Verifica APICASH_STELLAR_ISSUER_SECRET
        alt mainnet sem secret
            Anchor-->>Anchor: Erro crítico
        else testnet sem secret
            Anchor-->>Anchor: Skip (contas pré-fundadas)
        else secret presente
            Anchor->>CLI: BRLx SAC transfer issuer→buyer (stroops)
            CLI->>RPC: tx assinada
            RPC-->>CLI: tx_hash_issue
            Anchor->>CLI: BRLx SAC transfer buyer→escrow (stroops)
            CLI->>RPC: tx assinada
            RPC-->>CLI: tx_hash_transfer
        end
    end

    rect rgb(20,40,80)
        Note over Anchor,RPC: Subfluxo B — Lock no Contrato Escrow
        Anchor->>Custody: lock_funds(order_id, buyer, seller, amount)
        Custody->>Custody: Gera order_key u64 (hash do UUID)
        Custody->>CLI: ESCROW lock(order_key, buyer, seller, token, amount)
        CLI->>RPC: tx assinada
        RPC-->>CLI: soroban_lock_tx_hash
        Custody->>Custody: Custody { status=Locked, ttl=7d }
    end

    rect rgb(20,70,40)
        Note over Anchor,RPC: Subfluxo C — Release + Distribuição de Yield
        Anchor->>Custody: release_funds(order_id, released_by)
        Custody->>Custody: yield = amount × days / 1_000_000
        Note right of Custody: buyer 70% / seller 10% / platform 20%
        Custody->>CLI: ESCROW release(order_key, buyer, seller)
        CLI->>RPC: tx assinada
        RPC-->>CLI: soroban_release_tx_hash
        Custody->>Custody: Custody { status=Released, yield_earned }
    end
        </div>
      </div>
    </div>

  </main>
  <script>
    mermaid.initialize({ startOnLoad: true, theme: 'dark', securityLevel: 'loose' });

    function show(idx, btn) {
      document.querySelectorAll('.flow').forEach((el, i) => {
        el.classList.toggle('visible', i === idx);
      });
      document.querySelectorAll('nav button').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    }
  </script>
</body>
</html>"#;

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
  <title>APICash API Docs</title>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
  <style>
    body { margin: 0; }
    #swagger-ui .topbar { background-color: #1a1a2e; }
    #swagger-ui .topbar .download-url-wrapper { display: none; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.onload = function () {
      SwaggerUIBundle({
        url: "/openapi.json",
        dom_id: '#swagger-ui',
        deepLinking: true,
        presets: [SwaggerUIBundle.presets.apis],
        layout: "BaseLayout",
        requestCredentials: "include"
      });
    };
  </script>
</body>
</html>"#;
