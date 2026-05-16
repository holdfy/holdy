use sha2::{Digest, Sha256};

pub fn payload_hash_hex(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hex::encode(hasher.finalize())
}

/// Canonical payload for pix_tx (matches Go PayloadHashPixTx).
#[derive(serde::Serialize)]
struct CanonicalPayloadPixTx<'a> {
    entity_type: &'a str,
    entity_id: &'a str,
    amount: f64,
    end_to_end_id: &'a str,
    occurred_at: &'a str,
    account_id: i64,
    status: &'a str,
}

/// PayloadHashPixTx builds canonical hash for pix_tx entity (for anchor).
pub fn payload_hash_pix_tx(
    entity_id: &str,
    amount: f64,
    end_to_end_id: &str,
    occurred_at: &str,
    account_id: i64,
    status: &str,
) -> String {
    let p = CanonicalPayloadPixTx {
        entity_type: "pix_tx",
        entity_id,
        amount,
        end_to_end_id,
        occurred_at,
        account_id,
        status,
    };
    let json = serde_json::to_string(&p).unwrap_or_default();
    format!("0x{}", payload_hash_hex(json.as_bytes()))
}
