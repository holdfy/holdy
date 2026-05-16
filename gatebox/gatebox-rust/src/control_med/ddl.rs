pub const SQL_LIST: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, bank_id, endtoend, details,
           status_controle_med_id, amount, data_med, deleted_at
    FROM control_med ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, bank_id, endtoend, details,
           status_controle_med_id, amount, data_med, deleted_at
    FROM control_med WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO control_med (account_id, invoice_id, partners_id, bank_id, endtoend, details,
        status_controle_med_id, amount, data_med, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE control_med SET account_id = $1, invoice_id = $2, partners_id = $3, bank_id = $4,
        endtoend = $5, details = $6, status_controle_med_id = $7, amount = $8, data_med = $9, deleted_at = $10
    WHERE id = $11
"#;
pub const SQL_DELETE: &str = "DELETE FROM control_med WHERE id = $1";
