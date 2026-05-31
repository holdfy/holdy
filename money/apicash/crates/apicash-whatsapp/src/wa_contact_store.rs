//! Persistência de contatos WhatsApp (peer_key → nome + CPF/CNPJ).

use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WaContact {
    pub peer_key: String,
    pub user_id: Uuid,
    pub name: Option<String>,
    pub document: Option<String>,
    pub document_type: Option<String>,
    pub situation: Option<String>,
}

/// Grava (ou atualiza) dados de contato. Usa UPSERT: nunca sobrescreve nome/CPF já preenchido
/// por valor nulo — só atualiza quando o novo valor for não-nulo.
pub async fn save_contact(pool: &PgPool, c: &WaContact) {
    let doc_type = c.document.as_deref().map(|d| {
        if d.len() == 11 { "cpf" } else { "cnpj" }
    });
    let _ = sqlx::query(
        "INSERT INTO wa_contacts (peer_key, user_id, name, document, document_type, situation)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (peer_key) DO UPDATE SET
             name          = COALESCE(EXCLUDED.name,          wa_contacts.name),
             document      = COALESCE(EXCLUDED.document,      wa_contacts.document),
             document_type = COALESCE(EXCLUDED.document_type, wa_contacts.document_type),
             situation     = COALESCE(EXCLUDED.situation,     wa_contacts.situation),
             updated_at    = NOW()",
    )
    .bind(&c.peer_key)
    .bind(c.user_id)
    .bind(&c.name)
    .bind(&c.document)
    .bind(doc_type.or(c.document_type.as_deref()))
    .bind(&c.situation)
    .execute(pool)
    .await;
}

/// Carrega contato por peer_key. Retorna `None` se não encontrado ou em erro.
pub async fn load_contact(pool: &PgPool, peer_key: &str) -> Option<WaContact> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT peer_key, user_id, name, document, document_type, situation
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
    })
}
