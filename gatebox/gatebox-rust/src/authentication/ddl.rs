pub const SQL_LIST: &str = r#"
    SELECT id, name, username, password, type_auth_id, active, force_reset,
           created_at AT TIME ZONE 'UTC' AS created_at,
           updated_at AT TIME ZONE 'UTC' AS updated_at,
           deleted_at AT TIME ZONE 'UTC' AS deleted_at
    FROM authentication ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, name, username, password, type_auth_id, active, force_reset,
           created_at AT TIME ZONE 'UTC' AS created_at,
           updated_at AT TIME ZONE 'UTC' AS updated_at,
           deleted_at AT TIME ZONE 'UTC' AS deleted_at
    FROM authentication WHERE id = $1
"#;
pub const SQL_GET_BY_USERNAME: &str = r#"
    SELECT id, name, username, password, type_auth_id, active, force_reset,
           created_at AT TIME ZONE 'UTC' AS created_at,
           updated_at AT TIME ZONE 'UTC' AS updated_at,
           deleted_at AT TIME ZONE 'UTC' AS deleted_at
    FROM authentication WHERE username = $1 AND deleted_at IS NULL LIMIT 1
"#;
pub const SQL_GET_BY_USERNAME_AND_TYPE: &str = r#"
    SELECT id, name, username, password, type_auth_id, active, force_reset,
           created_at AT TIME ZONE 'UTC' AS created_at,
           updated_at AT TIME ZONE 'UTC' AS updated_at,
           deleted_at AT TIME ZONE 'UTC' AS deleted_at
    FROM authentication WHERE username = $1 AND type_auth_id = $2 AND deleted_at IS NULL LIMIT 1
"#;
pub const SQL_UPDATE_PASSWORD: &str = r#"
    UPDATE authentication SET password = $1, updated_at = NOW() WHERE id = $2
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO authentication (name, username, password, type_auth_id, active, force_reset, created_at, updated_at, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE authentication SET name = $1, username = $2, password = $3, type_auth_id = $4, active = $5,
        force_reset = $6, created_at = $7, updated_at = $8, deleted_at = $9 WHERE id = $10
"#;
pub const SQL_DELETE: &str = "DELETE FROM authentication WHERE id = $1";
