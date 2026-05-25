//! Persistência de conversas WhatsApp no MongoDB.
//!
//! Coleções:
//!   • `wa_messages`             — cada mensagem trocada (inbound + outbound)
//!   • `wa_conversation_summaries` — resumos gerados pelo LLM em momentos-chave
//!
//! Ativado por: `MONGODB_URL` (ex. `mongodb://localhost:27017`) + `MONGODB_DB` (padrão `apicash_wa`).
//! Se as variáveis não estiverem definidas, o store opera em modo no-op e não falha.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use mongodb::bson::{self, doc};
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

// ---- tipos públicos --------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

/// Uma mensagem trocada no canal WhatsApp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaMessage {
    /// Número WhatsApp normalizado (só dígitos).
    pub session_key: String,
    pub user_id: String,
    pub direction: MessageDirection,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub order_id: Option<String>,
    /// Nome do estado da sessão no momento da mensagem (ex. "Idle", "AwaitingPayment").
    pub flow_state_tag: String,
    /// ID da mensagem no WhatsApp (quando disponível).
    pub message_id: Option<String>,
}

/// Resumo gerado pelo LLM para auxiliar disputas futuras.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaConversationSummary {
    pub session_key: String,
    pub user_id: String,
    pub order_id: Option<String>,
    /// Evento que disparou o resumo.
    pub trigger: SummaryTrigger,
    pub summary: String,
    pub message_count: u32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SummaryTrigger {
    OrderCreated,
    PaymentConfirmed,
    DisputeOpened,
    SessionEnded,
}

// ---- store -----------------------------------------------------------------

pub struct ConversationStore {
    db: Option<Database>,
    /// Buffer das últimas mensagens da sessão corrente (para gerar resumo).
    /// Chave: session_key, Valor: vetor de mensagens recentes (max 50).
    buffer: RwLock<std::collections::HashMap<String, Vec<WaMessage>>>,
}

impl ConversationStore {
    /// Conecta ao MongoDB se `MONGODB_URL` estiver definido; caso contrário retorna store no-op.
    pub async fn from_env() -> Arc<Self> {
        let url = std::env::var("MONGODB_URL")
            .or_else(|_| std::env::var("MONGO_URL"))
            .unwrap_or_default();
        let db_name = std::env::var("MONGODB_DB").unwrap_or_else(|_| "apicash_wa".to_string());

        if url.is_empty() {
            info!("conversation_store: MONGODB_URL ausente — modo no-op (conversas não persistidas)");
            return Arc::new(Self { db: None, buffer: Default::default() });
        }

        match Client::with_uri_str(&url).await {
            Ok(client) => {
                let db = client.database(&db_name);
                info!(%url, %db_name, "conversation_store: conectado ao MongoDB");
                // Criar índices de background (fire-and-forget)
                let db_clone = db.clone();
                tokio::spawn(async move { ensure_indexes(&db_clone).await });
                Arc::new(Self { db: Some(db), buffer: Default::default() })
            }
            Err(e) => {
                warn!(error = %e, "conversation_store: falha ao conectar MongoDB — modo no-op");
                Arc::new(Self { db: None, buffer: Default::default() })
            }
        }
    }

    /// Registra uma mensagem (inbound ou outbound).
    pub async fn record_message(&self, msg: WaMessage) {
        // Atualizar buffer em memória (para gerar resumo posterior)
        {
            let mut buf = self.buffer.write().await;
            let entry = buf.entry(msg.session_key.clone()).or_default();
            entry.push(msg.clone());
            if entry.len() > 50 {
                entry.remove(0);
            }
        }

        let Some(db) = &self.db else { return };
        let col: Collection<bson::Document> = db.collection("wa_messages");
        let doc = match bson::to_document(&msg) {
            Ok(d) => d,
            Err(e) => { warn!(error = %e, "conversation_store: bson serialize error"); return; }
        };
        if let Err(e) = col.insert_one(doc).await {
            warn!(error = %e, "conversation_store: insert_one wa_messages failed");
        }
    }

    /// Gera resumo LLM das mensagens em buffer e persiste na coleção `wa_conversation_summaries`.
    pub async fn generate_and_save_summary(
        &self,
        session_key: &str,
        user_id: Uuid,
        order_id: Option<Uuid>,
        trigger: SummaryTrigger,
    ) {
        let messages = {
            let buf = self.buffer.read().await;
            buf.get(session_key).cloned().unwrap_or_default()
        };

        if messages.is_empty() { return; }

        let message_count = messages.len() as u32;
        let period_start = messages.first().map(|m| m.timestamp).unwrap_or_else(Utc::now);
        let period_end = messages.last().map(|m| m.timestamp).unwrap_or_else(Utc::now);

        let summary_text = call_llm_summary(&messages).await;

        let summary = WaConversationSummary {
            session_key: session_key.to_string(),
            user_id: user_id.to_string(),
            order_id: order_id.map(|id| id.to_string()),
            trigger,
            summary: summary_text,
            message_count,
            period_start,
            period_end,
            created_at: Utc::now(),
        };

        let Some(db) = &self.db else { return };
        let col: Collection<bson::Document> = db.collection("wa_conversation_summaries");
        let doc = match bson::to_document(&summary) {
            Ok(d) => d,
            Err(e) => { warn!(error = %e, "conversation_store: bson summary serialize error"); return; }
        };
        if let Err(e) = col.insert_one(doc).await {
            warn!(error = %e, "conversation_store: insert_one wa_conversation_summaries failed");
        } else {
            info!(session_key, trigger = ?summary.trigger, message_count, "conversation_store: summary saved");
        }
    }

    /// Limpar buffer de uma sessão (após idle longo, por exemplo).
    pub async fn clear_buffer(&self, session_key: &str) {
        self.buffer.write().await.remove(session_key);
    }
}

// ---- índices ---------------------------------------------------------------

async fn ensure_indexes(db: &Database) {
    use mongodb::IndexModel;
    use mongodb::options::IndexOptions;

    let col_msgs: Collection<bson::Document> = db.collection("wa_messages");
    let _ = col_msgs.create_index(
        IndexModel::builder()
            .keys(doc! { "session_key": 1, "timestamp": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build()
    ).await;

    let col_sums: Collection<bson::Document> = db.collection("wa_conversation_summaries");
    let _ = col_sums.create_index(
        IndexModel::builder()
            .keys(doc! { "session_key": 1, "created_at": -1 })
            .options(IndexOptions::builder().background(true).build())
            .build()
    ).await;
}

// ---- geração de resumo via LLM ---------------------------------------------

async fn call_llm_summary(messages: &[WaMessage]) -> String {
    let api_key = match std::env::var("OPENAI_API_KEY").ok().filter(|s| !s.is_empty()) {
        Some(k) => k,
        None => return build_fallback_summary(messages),
    };

    let transcript: String = messages
        .iter()
        .map(|m| {
            let dir = match m.direction {
                MessageDirection::Inbound => "Usuário",
                MessageDirection::Outbound => "Bot",
            };
            format!("[{}] {}: {}", m.timestamp.format("%H:%M"), dir, m.body)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "Você é um assistente jurídico. Resuma a conversa de WhatsApp abaixo em português, \
         focando em: (1) produto negociado, (2) valor combinado, (3) status do pagamento, \
         (4) problemas ou disputas mencionados. Seja objetivo, máximo 150 palavras.\n\n{transcript}"
    );

    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 300,
        "messages": [{ "role": "user", "content": prompt }]
    });

    let client = reqwest::Client::new();
    let Ok(resp) = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()
        .await
    else {
        return build_fallback_summary(messages);
    };

    let Ok(json) = resp.json::<serde_json::Value>().await else {
        return build_fallback_summary(messages);
    };

    json.get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string()
        .trim()
        .to_string()
        .pipe_if_empty(|| build_fallback_summary(messages))
}

fn build_fallback_summary(messages: &[WaMessage]) -> String {
    format!(
        "Conversa com {} mensagens ({} inbound, {} outbound).",
        messages.len(),
        messages.iter().filter(|m| m.direction == MessageDirection::Inbound).count(),
        messages.iter().filter(|m| m.direction == MessageDirection::Outbound).count(),
    )
}

trait PipeIfEmpty {
    fn pipe_if_empty(self, f: impl FnOnce() -> String) -> String;
}

impl PipeIfEmpty for String {
    fn pipe_if_empty(self, f: impl FnOnce() -> String) -> String {
        if self.is_empty() { f() } else { self }
    }
}
