// List transaction_anchors with optional filters (from, to, entity_type, period_type, period_id).
// Uses nullable params: $1..$5 for filters, $6 limit, $7 offset.
pub const SQL_LIST: &str = r#"
    SELECT id, idempotency_key, entity_type, entity_id, payload_hash,
           period_type, period_id, tx_hash, block_number, chain_id,
           anchored_at, dry_run, error_message, account_id, created_at
    FROM transaction_anchors
    WHERE ($1::timestamptz IS NULL OR created_at >= $1)
      AND ($2::timestamptz IS NULL OR created_at <= $2)
      AND ($3::text IS NULL OR entity_type = $3)
      AND ($4::text IS NULL OR period_type = $4)
      AND ($5::text IS NULL OR period_id = $5)
    ORDER BY created_at DESC
    LIMIT $6 OFFSET $7
"#;

pub const SQL_COUNT: &str = r#"
    SELECT COUNT(*) FROM transaction_anchors
    WHERE ($1::timestamptz IS NULL OR created_at >= $1)
      AND ($2::timestamptz IS NULL OR created_at <= $2)
      AND ($3::text IS NULL OR entity_type = $3)
      AND ($4::text IS NULL OR period_type = $4)
      AND ($5::text IS NULL OR period_id = $5)
"#;
