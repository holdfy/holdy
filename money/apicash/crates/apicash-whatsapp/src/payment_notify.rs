//! Registo de partes por `order_id` e parsing de referências de pagamento (QR Gatebox / banco).

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPaymentParties {
    pub seller_peer: String,
    pub buyer_peer: String,
    pub amount: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedPaymentNotify {
    parties: HashMap<Uuid, OrderPaymentParties>,
    notified: HashSet<Uuid>,
}

#[derive(Clone)]
pub struct PaymentNotifyRegistry {
    inner: Arc<RwLock<HashMap<Uuid, OrderPaymentParties>>>,
    notified: Arc<RwLock<HashSet<Uuid>>>,
    path: PathBuf,
}

impl PaymentNotifyRegistry {
    pub fn new() -> Self {
        Self::with_path(default_parties_path())
    }

    pub fn with_path(path: PathBuf) -> Self {
        let loaded = load_from_disk(&path);
        Self {
            inner: Arc::new(RwLock::new(loaded.parties)),
            notified: Arc::new(RwLock::new(loaded.notified)),
            path,
        }
    }

    pub async fn register(&self, order_id: Uuid, parties: OrderPaymentParties) {
        self.inner.write().await.insert(order_id, parties);
        let _ = self.persist().await;
    }

    pub async fn get(&self, order_id: Uuid) -> Option<OrderPaymentParties> {
        self.inner.read().await.get(&order_id).cloned()
    }

    /// Retorna o order_id mais recente (último inserido) onde este peer é o vendedor.
    /// Usado quando a sessão foi reiniciada e active_order_id foi perdido.
    pub async fn find_order_for_seller(&self, seller_peer: &str) -> Option<(Uuid, OrderPaymentParties)> {
        let inner = self.inner.read().await;
        // Itera todas as entradas — a coleção é pequena (dezenas de pedidos no máximo).
        // Sem timestamp no registry, devolve o primeiro match encontrado.
        inner
            .iter()
            .find(|(_, p)| p.seller_peer == seller_peer)
            .map(|(id, p)| (*id, p.clone()))
    }

    pub async fn was_notified(&self, order_id: Uuid) -> bool {
        self.notified.read().await.contains(&order_id)
    }

    pub async fn mark_notified(&self, order_id: Uuid) {
        if self.notified.write().await.insert(order_id) {
            let _ = self.persist().await;
        }
    }

    async fn persist(&self) -> Result<(), String> {
        let parties = self.inner.read().await.clone();
        let notified = self.notified.read().await.clone();
        let data = PersistedPaymentNotify { parties, notified };
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        tokio::fs::write(&self.path, json)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Extrai UUID do pedido a partir do QR / referência de pagamento.
    #[must_use]
    pub fn parse_order_id_from_reference(reference: &str) -> Option<Uuid> {
        let r = reference.trim();
        if let Some(rest) = r.strip_prefix("GATEBOXRUST:QR|") {
            let token = rest.split('|').next()?.trim();
            if let Some(id) = token.strip_prefix("order_") {
                return Uuid::parse_str(id).ok();
            }
        }
        if let Some(pos) = r.find("order:") {
            let tail = &r[pos + "order:".len()..];
            let id_str: String = tail
                .chars()
                .take_while(|c| *c == '-' || c.is_ascii_hexdigit())
                .collect();
            return Uuid::parse_str(&id_str).ok();
        }
        if let Some(pos) = r.find("order_") {
            let tail = &r[pos + "order_".len()..];
            let id_str: String = tail
                .chars()
                .take_while(|c| *c == '-' || c.is_ascii_hexdigit())
                .collect();
            return Uuid::parse_str(&id_str).ok();
        }
        None
    }
}

fn default_parties_path() -> PathBuf {
    if let Ok(p) = std::env::var("APICASH_WA_ORDER_PARTIES_PATH") {
        if !p.trim().is_empty() {
            return PathBuf::from(p.trim());
        }
    }
    if let Ok(uri) = std::env::var("APICASH_WA_SQLITE_PATH") {
        if let Some(db) = crate::wa_peer::sqlite_path_from_uri(&uri) {
            return db
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("wa_order_parties.json");
        }
    }
    PathBuf::from(".runapp/wa_order_parties.json")
}

fn load_from_disk(path: &PathBuf) -> PersistedPaymentNotify {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => PersistedPaymentNotify::default(),
    }
}
