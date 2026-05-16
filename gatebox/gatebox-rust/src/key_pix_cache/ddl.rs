pub const SQL_LIST: &str = r#"
    SELECT id, key, pix_key_type_id, document_number, description, bank_name, account_number, branch, ispb, hide_document, deleted_at
    FROM key_pix_cache ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, key, pix_key_type_id, document_number, description, bank_name, account_number, branch, ispb, hide_document, deleted_at
    FROM key_pix_cache WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO key_pix_cache (key, pix_key_type_id, document_number, description, bank_name, account_number, branch, ispb, hide_document, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE key_pix_cache SET key = $1, pix_key_type_id = $2, document_number = $3, description = $4, bank_name = $5, account_number = $6, branch = $7, ispb = $8, hide_document = $9, deleted_at = $10
    WHERE id = $11
"#;
pub const SQL_DELETE: &str = "DELETE FROM key_pix_cache WHERE id = $1";
