pub const SQL_INSERT: &str = r#"
    INSERT INTO disputes (transaction_id, account_id, type, status, reason, evidence, created_at)
    VALUES ($1, $2, $3, 'OPEN', $4, $5, NOW())
    RETURNING id
"#;

pub const SQL_LIST: &str = r#"
    SELECT id, transaction_id, account_id, type, status, reason, evidence,
           created_at, resolved_at, resolved_by, resolution_notes,
           COUNT(*) OVER() AS full_count
    FROM disputes
    WHERE (NULLIF($1, '') IS NULL OR status = $1)
    ORDER BY created_at DESC
    LIMIT $2 OFFSET $3
"#;

pub const SQL_LIST_BY_ACCOUNT: &str = r#"
    SELECT id, transaction_id, account_id, type, status, reason, evidence,
           created_at, resolved_at, resolved_by, resolution_notes,
           COUNT(*) OVER() AS full_count
    FROM disputes
    WHERE account_id = $1
    ORDER BY created_at DESC
    LIMIT $2 OFFSET $3
"#;

pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, transaction_id, account_id, type, status, reason, evidence,
           created_at, resolved_at, resolved_by, resolution_notes, NULL::BIGINT AS full_count
    FROM disputes WHERE id = $1
"#;

pub const SQL_RESOLVE: &str = r#"
    UPDATE disputes
    SET status = 'RESOLVED',
        resolved_at = NOW(),
        resolved_by = $2,
        resolution_notes = $3
    WHERE id = $1 AND status = 'OPEN'
"#;
