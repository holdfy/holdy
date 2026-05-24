pub const SQL_LIST: &str = r#"
    SELECT id, level, service, message, created_at
    FROM app_log
    WHERE (NULLIF($1, '') IS NULL OR level = $1)
      AND (NULLIF($2, '') IS NULL OR service = $2)
    ORDER BY created_at DESC
    LIMIT $3 OFFSET $4
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO app_log (level, service, message, created_at)
    VALUES ($1, $2, $3, NOW())
    RETURNING id
"#;
