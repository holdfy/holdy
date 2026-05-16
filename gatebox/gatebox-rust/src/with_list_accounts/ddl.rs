pub const SQL_LIST: &str = r#"
    SELECT id, type_external_id, account_id, document
    FROM with_list_accounts ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, type_external_id, account_id, document
    FROM with_list_accounts WHERE id = $1
"#;
/// type_external_id = 1 = PIX_OUT
pub const SQL_IS_WHITELISTED_PIX_OUT: &str = r#"
    SELECT COUNT(*) > 0 FROM with_list_accounts
    WHERE account_id = $1 AND (type_external_id = 1 OR type_external_id IS NULL)
"#;
/// type_external_id = 2 = PIX_IN
pub const SQL_IS_WHITELISTED_PIX_IN: &str = r#"
    SELECT COUNT(*) > 0 FROM with_list_accounts
    WHERE account_id = $1 AND (type_external_id = 2 OR type_external_id IS NULL)
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO with_list_accounts (type_external_id, account_id, document)
    VALUES ($1, $2, $3) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE with_list_accounts SET type_external_id = $1, account_id = $2, document = $3
    WHERE id = $4
"#;
pub const SQL_DELETE: &str = "DELETE FROM with_list_accounts WHERE id = $1";
