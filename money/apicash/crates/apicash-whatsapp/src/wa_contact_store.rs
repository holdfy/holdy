//! Persistência de contatos WhatsApp (peer_key → nome + CPF/CNPJ)
//! e cache de consultas NFS-e (document → nome + situação).
//!
//! Cache hierarchy: Redis (24h) → Postgres `nfse_document_cache` → NFS-e portal.

use redis::{aio::ConnectionManager, AsyncCommands};
use sqlx::PgPool;
use uuid::Uuid;

const KYC_REDIS_KEY_PREFIX: &str = "nfse:v1:";
const KYC_REDIS_TTL_SECS: u64 = 86_400;

// ─── Redis helpers (NFS-e cache layer 1) ─────────────────────────────────────

/// Consulta Redis para CPF/CNPJ (só dígitos). Retorna None se ausente ou erro.
pub async fn get_nfse_redis(
    conn: &mut ConnectionManager,
    document_digits: &str,
) -> Option<crate::nfse_client::PersonLookup> {
    let key = format!("{KYC_REDIS_KEY_PREFIX}{document_digits}");
    let raw: Option<String> = conn.get(&key).await.ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw?).ok()?;
    let doc_label: &'static str = if document_digits.len() == 11 { "CPF" } else { "CNPJ" };
    Some(crate::nfse_client::PersonLookup {
        document_digits: document_digits.to_string(),
        document_label: doc_label,
        name: v.get("n").and_then(|x| x.as_str()).map(|s| s.to_string()),
        situation: v.get("s").and_then(|x| x.as_str()).map(|s| s.to_string()),
    })
}

/// Grava resultado de consulta NFS-e no Redis (TTL 24h). Só persiste quando há nome.
pub async fn set_nfse_redis(conn: &mut ConnectionManager, person: &crate::nfse_client::PersonLookup) {
    if person.name.is_none() {
        return;
    }
    let key = format!("{KYC_REDIS_KEY_PREFIX}{}", person.document_digits);
    let payload = serde_json::json!({
        "n": person.name,
        "s": person.situation,
    });
    let _ = conn
        .set_ex::<_, _, ()>(&key, payload.to_string(), KYC_REDIS_TTL_SECS)
        .await;
}

// ─── Cache NFS-e (Receita Federal) ───────────────────────────────────────────

/// Retorna dados cacheados da Receita Federal para um CPF/CNPJ (só dígitos).
/// Retorna `None` se não houver cache ou se o cache não tiver nome.
pub async fn get_nfse_cache(
    pool: &PgPool,
    document_digits: &str,
) -> Option<crate::nfse_client::PersonLookup> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT document, document_type, name, situation
         FROM nfse_document_cache WHERE document = $1 AND name IS NOT NULL",
    )
    .bind(document_digits)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    let doc_label: &'static str = if document_digits.len() == 11 { "CPF" } else { "CNPJ" };
    Some(crate::nfse_client::PersonLookup {
        document_digits: document_digits.to_string(),
        document_label: doc_label,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        situation: row.try_get::<Option<String>, _>("situation").ok().flatten(),
    })
}

/// Grava resultado de consulta NFS-e no cache. Só persiste quando há nome.
pub async fn save_nfse_cache(pool: &PgPool, person: &crate::nfse_client::PersonLookup) {
    if person.name.is_none() {
        return;
    }
    let doc_type = if person.document_digits.len() == 11 { "cpf" } else { "cnpj" };
    let _ = sqlx::query(
        "INSERT INTO nfse_document_cache (document, document_type, name, situation)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (document) DO UPDATE SET
             name      = EXCLUDED.name,
             situation = COALESCE(EXCLUDED.situation, nfse_document_cache.situation),
             cached_at = NOW()",
    )
    .bind(&person.document_digits)
    .bind(doc_type)
    .bind(&person.name)
    .bind(&person.situation)
    .execute(pool)
    .await;
}

#[derive(Debug, Clone)]
pub struct WaContact {
    pub peer_key: String,
    pub user_id: Uuid,
    pub name: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub situation: Option<String>,
    pub pix_key: Option<String>,
}

/// Grava (ou atualiza) dados de contato. Usa UPSERT: nunca sobrescreve nome/CPF já preenchido
/// por valor nulo — só atualiza quando o novo valor for não-nulo.
pub async fn save_contact(pool: &PgPool, c: &WaContact) {
    let doc_type = c.document.as_deref().map(|d| {
        if d.len() == 11 { "cpf" } else { "cnpj" }
    });
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (peer_key, user_id, name, document, document_type, situation, pix_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (peer_key) DO UPDATE SET
             name          = COALESCE(EXCLUDED.name,          wa_contacts.name),
             document      = COALESCE(EXCLUDED.document,      wa_contacts.document),
             document_type = COALESCE(EXCLUDED.document_type, wa_contacts.document_type),
             situation     = COALESCE(EXCLUDED.situation,     wa_contacts.situation),
             pix_key       = COALESCE(EXCLUDED.pix_key,       wa_contacts.pix_key),
             updated_at    = NOW()",
    )
    .bind(&c.peer_key)
    .bind(c.user_id)
    .bind(&c.name)
    .bind(&c.document)
    .bind(doc_type.or(c.document_type.as_deref()))
    .bind(&c.situation)
    .bind(&c.pix_key)
    .execute(pool)
    .await;
}

/// Grava ou atualiza só a chave PIX.
/// Usa UPSERT para garantir que a linha seja criada mesmo que o vendedor
/// ainda não tenha passado pelo fluxo de documento (sem linha prévia em wa_contacts).
pub async fn save_pix_key(pool: &PgPool, peer_key: &str, pix_key: &str) {
    let user_id = crate::session::user_id_for_peer_key(peer_key);
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (peer_key, user_id, pix_key)
         VALUES ($1, $2, $3)
         ON CONFLICT (peer_key) DO UPDATE SET
             pix_key    = EXCLUDED.pix_key,
             updated_at = NOW()",
    )
    .bind(peer_key)
    .bind(user_id)
    .bind(pix_key)
    .execute(pool)
    .await;
}

/// Retorna a chave PIX guardada para um peer, ou `None`.
pub async fn load_pix_key(pool: &PgPool, peer_key: &str) -> Option<String> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT pix_key FROM wa_contacts WHERE peer_key = $1",
    )
    .bind(peer_key)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;
    row.try_get::<Option<String>, _>("pix_key").ok().flatten()
}

/// Carrega contato por peer_key. Retorna `None` se não encontrado ou em erro.
pub async fn load_contact(pool: &PgPool, peer_key: &str) -> Option<WaContact> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT peer_key, user_id, name, document, document_type, situation, pix_key
         FROM wa_contacts WHERE peer_key = $1",
    )
    .bind(peer_key)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()?;

    Some(WaContact {
        peer_key: row.try_get("peer_key").ok()?,
        user_id: row.try_get("user_id").ok()?,
        name: row.try_get("name").ok().flatten(),
        document: row.try_get("document").ok().flatten(),
        document_type: row.try_get("document_type").ok().flatten(),
        situation: row.try_get("situation").ok().flatten(),
        pix_key: row.try_get("pix_key").ok().flatten(),
    })
}
