pub const SQL_LIST: &str = r#"
    SELECT id, description, token, expire_in, authentication_id, timestamp, active
    FROM token_service ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, description, token, expire_in, authentication_id, timestamp, active
    FROM token_service WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO token_service (description, token, expire_in, authentication_id, timestamp, active)
    VALUES ($1, $2, $3, $4, $5, $6) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE token_service SET description = $1, token = $2, expire_in = $3, authentication_id = $4, timestamp = $5, active = $6
    WHERE id = $7
"#;
pub const SQL_DELETE: &str = "DELETE FROM token_service WHERE id = $1";
