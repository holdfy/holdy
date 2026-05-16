pub const SQL_LIST: &str = r#"
    SELECT id, callback_url, username, password, api_key, webhook_type_id, account_id, deleted_at
    FROM webhook_manager ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, callback_url, username, password, api_key, webhook_type_id, account_id, deleted_at
    FROM webhook_manager WHERE id = $1
"#;
pub const SQL_LIST_BY_ACCOUNT: &str = r#"
    SELECT id, callback_url, username, password, api_key, webhook_type_id, account_id, deleted_at
    FROM webhook_manager WHERE account_id = $1 AND deleted_at IS NULL ORDER BY id LIMIT $2 OFFSET $3
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO webhook_manager (callback_url, username, password, api_key, webhook_type_id, account_id, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE webhook_manager SET callback_url = $1, username = $2, password = $3, api_key = $4, webhook_type_id = $5, account_id = $6, deleted_at = $7
    WHERE id = $8
"#;
pub const SQL_DELETE: &str = "DELETE FROM webhook_manager WHERE id = $1";
