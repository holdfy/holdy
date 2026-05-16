pub const SQL_LIST: &str = r#"
    SELECT id, key, pix_key_type_id, description, deleted_at
    FROM shared_key ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, key, pix_key_type_id, description, deleted_at
    FROM shared_key WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO shared_key (key, pix_key_type_id, description, deleted_at)
    VALUES ($1, $2, $3, $4) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE shared_key SET key = $1, pix_key_type_id = $2, description = $3, deleted_at = $4
    WHERE id = $5
"#;
pub const SQL_DELETE: &str = "DELETE FROM shared_key WHERE id = $1";
