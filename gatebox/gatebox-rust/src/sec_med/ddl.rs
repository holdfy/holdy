pub const SQL_LIST: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, apagar, transaction_id,
           status_sec_med_id, amount, scheduled_date, deleted_at
    FROM sec_med ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, apagar, transaction_id,
           status_sec_med_id, amount, scheduled_date, deleted_at
    FROM sec_med WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO sec_med (account_id, invoice_id, partners_id, apagar, transaction_id,
        status_sec_med_id, amount, scheduled_date, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE sec_med SET account_id = $1, invoice_id = $2, partners_id = $3, apagar = $4,
        transaction_id = $5, status_sec_med_id = $6, amount = $7, scheduled_date = $8, deleted_at = $9
    WHERE id = $10
"#;
pub const SQL_DELETE: &str = "DELETE FROM sec_med WHERE id = $1";
