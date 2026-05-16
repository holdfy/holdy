pub const SQL_LIST: &str = r#"
    SELECT id, account_id, receive_external, deposit_external, descricao
    FROM account_rules ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, account_id, receive_external, deposit_external, descricao
    FROM account_rules WHERE id = $1
"#;
pub const SQL_GET_BY_ACCOUNT_ID: &str = r#"
    SELECT id, account_id, receive_external, deposit_external, descricao
    FROM account_rules WHERE account_id = $1 LIMIT 1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO account_rules (account_id, receive_external, deposit_external, descricao)
    VALUES ($1, $2, $3, $4) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE account_rules SET account_id = $1, receive_external = $2, deposit_external = $3, descricao = $4
    WHERE id = $5
"#;
pub const SQL_DELETE: &str = "DELETE FROM account_rules WHERE id = $1";
