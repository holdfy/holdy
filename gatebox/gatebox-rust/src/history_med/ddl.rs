pub const SQL_LIST: &str = r#"
    SELECT id, account_id, control_med_id, sec_med_id, apagar, amount, deleted_at
    FROM history_med ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, account_id, control_med_id, sec_med_id, apagar, amount, deleted_at
    FROM history_med WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO history_med (account_id, control_med_id, sec_med_id, apagar, amount, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE history_med SET account_id = $1, control_med_id = $2, sec_med_id = $3, apagar = $4, amount = $5, deleted_at = $6
    WHERE id = $7
"#;
pub const SQL_DELETE: &str = "DELETE FROM history_med WHERE id = $1";
