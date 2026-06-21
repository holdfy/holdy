//! Auditoria de atividade web no MongoDB — paridade com `ConversationStore` do canal WhatsApp.
//!
//! Coleções:
//!   • `web_listing_imports` — cada importação de URL disparada pelo site
//!   • `web_order_events`    — criação de pedido, abertura de disputa e liberação de custódia via site
//!
//! Ativado pelo mesmo par de variáveis que o WhatsApp: `MONGODB_URL` + `MONGODB_DB` (padrão `apicash_wa`).
//! Modo no-op automático quando as variáveis não estão presentes — nunca falha o handler principal.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use mongodb::bson;
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

// ─── Eventos ─────────────────────────────────────────────────────────────────

/// Uma importação de produto disparada via site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebImportEvent {
    pub user_id: Option<String>,
    pub url: String,
    pub listing_id: Option<String>,
    pub title: String,
    pub source_platform: String,
    pub photos_count: usize,
    pub price_suggested: Option<String>,
    pub imported_at: DateTime<Utc>,
}

/// Tipo de evento no ciclo de vida de um pedido no site.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebOrderEventKind {
    OrderCreated,
    DisputeOpened,
    OrderReleased,
}

/// Evento de ciclo de vida de pedido registrado pelo site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebOrderEvent {
    pub user_id: Option<String>,
    pub order_id: String,
    pub amount: String,
    pub listing_id: Option<String>,
    pub kind: WebOrderEventKind,
    pub occurred_at: DateTime<Utc>,
}

// ─── Store ───────────────────────────────────────────────────────────────────

pub struct WebActivityStore {
    db: Option<Database>,
}

impl WebActivityStore {
    /// Conecta ao MongoDB usando as mesmas variáveis do `ConversationStore`.
    /// Retorna store no-op silencioso se `MONGODB_URL` não estiver definido.
    pub async fn from_env() -> Arc<Self> {
        let url = std::env::var("MONGODB_URL")
            .or_else(|_| std::env::var("MONGO_URL"))
            .unwrap_or_default();
        let db_name = std::env::var("MONGODB_DB").unwrap_or_else(|_| "apicash_wa".to_string());

        if url.is_empty() {
            return Arc::new(Self { db: None });
        }

        match Client::with_uri_str(&url).await {
            Ok(client) => {
                let db = client.database(&db_name);
                info!(%url, %db_name, "activity_store: conectado ao MongoDB");
                let db_clone = db.clone();
                tokio::spawn(async move { ensure_indexes(&db_clone).await });
                Arc::new(Self { db: Some(db) })
            }
            Err(e) => {
                warn!(error = %e, "activity_store: falha ao conectar MongoDB — modo no-op");
                Arc::new(Self { db: None })
            }
        }
    }

    /// Versão síncrona no-op para contextos sem async (testes, path de dev sem Postgres).
    pub fn noop() -> Arc<Self> {
        Arc::new(Self { db: None })
    }

    /// Registra uma importação de produto (fire-and-forget — não deve ser awaited).
    pub async fn record_import(&self, event: WebImportEvent) {
        let Some(db) = &self.db else { return };
        let col: Collection<bson::Document> = db.collection("web_listing_imports");
        let doc = match bson::to_document(&event) {
            Ok(d) => d,
            Err(e) => { warn!(error = %e, "activity_store: bson serialize web_import"); return; }
        };
        if let Err(e) = col.insert_one(doc).await {
            warn!(error = %e, "activity_store: insert web_listing_imports failed");
        }
    }

    /// Registra um evento de ciclo de vida de pedido (fire-and-forget — não deve ser awaited).
    pub async fn record_order_event(&self, event: WebOrderEvent) {
        let Some(db) = &self.db else { return };
        let col: Collection<bson::Document> = db.collection("web_order_events");
        let doc = match bson::to_document(&event) {
            Ok(d) => d,
            Err(e) => { warn!(error = %e, "activity_store: bson serialize web_order_event"); return; }
        };
        if let Err(e) = col.insert_one(doc).await {
            warn!(error = %e, "activity_store: insert web_order_events failed");
        }
    }
}

// ─── Índices ─────────────────────────────────────────────────────────────────

async fn ensure_indexes(db: &Database) {
    use mongodb::IndexModel;
    use mongodb::bson::doc;
    use mongodb::options::IndexOptions;

    let col_imports: Collection<bson::Document> = db.collection("web_listing_imports");
    let _ = col_imports.create_index(
        IndexModel::builder()
            .keys(doc! { "user_id": 1, "imported_at": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build(),
    ).await;

    let col_orders: Collection<bson::Document> = db.collection("web_order_events");
    let _ = col_orders.create_index(
        IndexModel::builder()
            .keys(doc! { "order_id": 1, "occurred_at": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build(),
    ).await;
}

// ─── Helper para spawn fire-and-forget ───────────────────────────────────────

/// Loga uma importação web em background sem bloquear o handler.
pub fn spawn_record_import(store: Arc<WebActivityStore>, user_id: Option<Uuid>, listing_id: Option<Uuid>, title: &str, source_platform: &str, photos_count: usize, price_suggested: Option<String>, url: &str) {
    let event = WebImportEvent {
        user_id: user_id.map(|u| u.to_string()),
        url: url.to_string(),
        listing_id: listing_id.map(|u| u.to_string()),
        title: title.to_string(),
        source_platform: source_platform.to_string(),
        photos_count,
        price_suggested,
        imported_at: Utc::now(),
    };
    tokio::spawn(async move { store.record_import(event).await });
}

/// Loga um evento de pedido web em background sem bloquear o handler.
pub fn spawn_record_order_event(store: Arc<WebActivityStore>, user_id: Option<Uuid>, order_id: Uuid, amount: &str, listing_id: Option<Uuid>, kind: WebOrderEventKind) {
    let event = WebOrderEvent {
        user_id: user_id.map(|u| u.to_string()),
        order_id: order_id.to_string(),
        amount: amount.to_string(),
        listing_id: listing_id.map(|u| u.to_string()),
        kind,
        occurred_at: Utc::now(),
    };
    tokio::spawn(async move { store.record_order_event(event).await });
}
