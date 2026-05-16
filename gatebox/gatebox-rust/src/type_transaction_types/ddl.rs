pub const SQL_LIST: &str = r#"
    SELECT id, code, description FROM type_transaction_types ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"SELECT id, code, description FROM type_transaction_types WHERE id = $1"#;
pub const SQL_INSERT: &str = r#"INSERT INTO type_transaction_types (code, description) VALUES ($1, $2) RETURNING id"#;
pub const SQL_UPDATE: &str = r#"UPDATE type_transaction_types SET code = $1, description = $2 WHERE id = $3"#;
pub const SQL_DELETE: &str = "DELETE FROM type_transaction_types WHERE id = $1";
