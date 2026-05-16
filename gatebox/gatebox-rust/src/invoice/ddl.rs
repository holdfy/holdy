pub const SQL_LIST: &str = r#"
    SELECT id, identifier, key, pix_key_type_id, invoice_type_id, timeout, expire,
           partners_list_id, amount, invoice_status_id, external_id, document_number,
           description, account_id, deleted_at
    FROM invoice ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, identifier, key, pix_key_type_id, invoice_type_id, timeout, expire,
           partners_list_id, amount, invoice_status_id, external_id, document_number,
           description, account_id, deleted_at
    FROM invoice WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO invoice (identifier, key, pix_key_type_id, invoice_type_id, timeout, expire,
        partners_list_id, amount, invoice_status_id, external_id, document_number, description, account_id, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE invoice SET identifier = $1, key = $2, pix_key_type_id = $3, invoice_type_id = $4,
        timeout = $5, expire = $6, partners_list_id = $7, amount = $8, invoice_status_id = $9,
        external_id = $10, document_number = $11, description = $12, account_id = $13, deleted_at = $14
    WHERE id = $15
"#;
pub const SQL_DELETE: &str = "DELETE FROM invoice WHERE id = $1";

/// Get invoice by external_id (for PIX IN webhook - gateway from QR Code).
pub const SQL_GET_BY_EXTERNAL_ID: &str = r#"
    SELECT i.id, i.invoice_type_id, i.partners_list_id, pl.description
    FROM invoice i
    LEFT JOIN partners_list pl ON pl.id = i.partners_list_id
    WHERE i.external_id = $1 AND (i.deleted_at IS NULL)
    LIMIT 1
"#;

/// Update invoice status (DONE = 2).
pub const SQL_UPDATE_STATUS: &str = r#"UPDATE invoice SET invoice_status_id = $1 WHERE id = $2"#;
