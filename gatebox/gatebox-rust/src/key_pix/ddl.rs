pub const SQL_LIST: &str = r#"
    SELECT id, key, pix_key_type_id, document_number, description, account_id, partners_id, deleted_at
    FROM key_pix ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, key, pix_key_type_id, document_number, description, account_id, partners_id, deleted_at
    FROM key_pix WHERE id = $1
"#;
pub const SQL_GET_BY_KEY: &str = r#"
    SELECT id, key, pix_key_type_id, document_number, description, account_id, partners_id, deleted_at
    FROM key_pix WHERE key = $1 AND deleted_at IS NULL LIMIT 1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO key_pix (key, pix_key_type_id, document_number, description, account_id, partners_id, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE key_pix SET key = $1, pix_key_type_id = $2, document_number = $3, description = $4, account_id = $5, partners_id = $6, deleted_at = $7
    WHERE id = $8
"#;
pub const SQL_DELETE: &str = "DELETE FROM key_pix WHERE id = $1";
