//! Normalização de identificadores de peer (Cloud API vs JID multi-device).

use std::path::{Path, PathBuf};
use std::str::FromStr;

use rusqlite::Connection;
use whatsapp_rust::Jid;

use crate::utils::masking::mask_whatsapp_peer;

/// Caminho filesystem a partir da URI `file:...` do `SqliteStore` (/WhatsApp SQLite).
#[must_use]
pub fn sqlite_path_from_uri(uri: &str) -> Option<PathBuf> {
    let p = uri.trim().strip_prefix("file:")?.trim();
    if p.to_ascii_lowercase().starts_with(":memory") {
        return None;
    }
    Some(PathBuf::from(p))
}

fn lid_to_phone_digits(db_path: &Path, lid_user: &str) -> Option<String> {
    let conn = Connection::open(db_path).ok()?;
    let mut stmt = conn
        .prepare(
            "SELECT phone_number FROM lid_pn_mapping WHERE lid = ? ORDER BY updated_at DESC LIMIT 1",
        )
        .ok()?;
    let pn: Result<String, _> = stmt.query_row([lid_user], |row| row.get(0));
    pn.ok().and_then(|s| {
        let d: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        (d.len() >= 10).then_some(d)
    })
}

/// Alinha `sender_id` ao mesmo formato das chaves de sessão onde guardámos apenas **dígitos PN**.
///
/// Mensagens recentes WhatsApp podem vir com **`{lid}@lid`** em vez de `{pn}@s.whatsapp.net`. O
/// próprio cliente grava o PN correspondente na tabela `lid_pn_mapping` dessa mesma base SQLite.
#[must_use]
pub fn canonical_session_peer_key(peer_raw: String, sqlite_uri: Option<&str>) -> String {
    let trimmed = peer_raw.trim().to_owned();

    if let Some((lid_user_raw, domain)) = trimmed.rsplit_once('@') {
        if domain.eq_ignore_ascii_case("lid") {
            let lid_user = lid_user_raw.trim();
            let Some(uri) = sqlite_uri else {
                tracing::warn!(lid=%lid_user, "whatsapp: @lid sem URI SQLite para resolver PN");
                return trimmed;
            };
            let Some(path) = sqlite_path_from_uri(uri) else {
                tracing::warn!(lid=%lid_user, "whatsapp: URI SQLite inválida para resolver @lid");
                return trimmed;
            };
            return match lid_to_phone_digits(&path, lid_user) {
                Some(pn) => {
                    tracing::info!(
                        lid = %lid_user,
                        pn_masked = %mask_whatsapp_peer(&pn),
                        "whatsapp: chave de sessão normalizada (LID → PN)"
                    );
                    pn
                }
                None => {
                    tracing::warn!(
                        lid = %lid_user,
                        db = %path.display(),
                        "whatsapp: não há entrada lid_pn_mapping; B pode ficar Idle até haver PN aprendido"
                    );
                    trimmed
                }
            };
        }
    }
    trimmed
}

/// Chave estável quando o servidor JID é `s.whatsapp.net`: só dígitos do PN.
#[must_use]
pub fn peer_key_from_jid(sender: &Jid) -> String {
    let server = sender.server.as_ref();
    if server == "s.whatsapp.net" {
        sender.user.chars().filter(|c| c.is_ascii_digit()).collect()
    } else {
        sender.to_string()
    }
}

/// Destino para `Client::send_message`: aceita `5511999...@s.whatsapp.net` ou só dígitos.
pub fn peer_to_jid(peer: &str) -> Result<Jid, String> {
    let p = peer.trim();
    if p.contains('@') {
        Jid::from_str(p).map_err(|e| e.to_string())
    } else {
        let digits: String = p.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return Err("peer vazio ou sem dígitos".into());
        }
        Jid::from_str(&format!("{digits}@s.whatsapp.net")).map_err(|e| e.to_string())
    }
}

/// Resolve JID de entrega via servidor WhatsApp (`is_on_whatsapp` + `get_user_info`).
/// Evita enviar para LID/PN errado quando só temos dígitos digitados ou do cartão de contacto.
pub async fn resolve_delivery_jid(
    client: &whatsapp_rust::Client,
    peer: &str,
) -> Result<Jid, String> {
    let p = peer.trim();
    if p.contains('@') {
        return peer_to_jid(p);
    }

    let pn = crate::handlers::holdfy::canonical_peer_key(p)
        .ok_or_else(|| format!("número inválido ({})", mask_whatsapp_peer(p)))?;

    match client.contacts().is_on_whatsapp(&[&pn]).await {
        Ok(results) => {
            if let Some(r) = results.first() {
                if r.is_registered {
                    tracing::info!(
                        pn = %mask_whatsapp_peer(&pn),
                        delivery = %r.jid,
                        "whatsapp: destino confirmado (is_on_whatsapp)"
                    );
                    if let Err(e) = client.contacts().get_user_info(&[r.jid.clone()]).await {
                        tracing::warn!(error = %e, "whatsapp: get_user_info após is_on_whatsapp falhou (ignorado)");
                    }
                    return Ok(r.jid.clone());
                }
                return Err(format!(
                    "o número {} não está registado no WhatsApp",
                    mask_whatsapp_peer(&pn)
                ));
            }
            tracing::warn!(
                pn = %mask_whatsapp_peer(&pn),
                "whatsapp: is_on_whatsapp sem resultado; usa PN"
            );
            peer_to_jid(&pn)
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                pn = %mask_whatsapp_peer(&pn),
                "whatsapp: is_on_whatsapp falhou; usa PN"
            );
            peer_to_jid(&pn)
        }
    }
}

/// Dígitos PN canónicos confirmados pelo WhatsApp (via `is_on_whatsapp`).
pub async fn canonical_whatsapp_peer_digits(
    client: &whatsapp_rust::Client,
    peer: &str,
) -> Option<String> {
    let jid = resolve_delivery_jid(client, peer).await.ok()?;
    let d: String = jid.user.chars().filter(|c| c.is_ascii_digit()).collect();
    (!d.is_empty()).then_some(d)
}

/// Fallback síncrono (sem consulta ao servidor) — preferir [`resolve_delivery_jid`].
pub fn peer_to_delivery_jid(peer: &str, _sqlite_uri: Option<&str>) -> Result<Jid, String> {
    let p = peer.trim();
    if p.contains('@') {
        return peer_to_jid(p);
    }
    let pn = crate::handlers::holdfy::canonical_peer_key(p).unwrap_or_else(|| {
        p.chars().filter(|c| c.is_ascii_digit()).collect()
    });
    peer_to_jid(&pn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_lid_resolves_when_row_exists() {
        let dir = tempfile::tempdir().expect("tmp");
        let db = dir.path().join("t.db");
        {
            let conn = Connection::open(&db).unwrap();
            conn.execute_batch(
                r"
                CREATE TABLE lid_pn_mapping (
                    lid TEXT NOT NULL,
                    phone_number TEXT NOT NULL,
                    created_at BIGINT NOT NULL,
                    learning_source TEXT NOT NULL,
                    updated_at BIGINT NOT NULL,
                    device_id INTEGER NOT NULL,
                    PRIMARY KEY (lid, device_id)
                );
                INSERT INTO lid_pn_mapping (lid, phone_number, created_at, learning_source, updated_at, device_id)
                VALUES ('111', '5511999887766', 0, 'test', 0, 0);
            ",
            )
            .unwrap();
        }
        let uri = format!("file:{}", db.display());
        let out = canonical_session_peer_key("111@lid".into(), Some(&uri));
        assert_eq!(out, "5511999887766");
    }

    #[test]
    fn canonical_non_lid_unchanged() {
        let uri = "file:./nope.db";
        assert_eq!(
            canonical_session_peer_key("5547123456789".into(), Some(uri)),
            "5547123456789"
        );
    }
}
