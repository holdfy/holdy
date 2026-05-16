pub const SQL_LIST: &str = r#"
    SELECT id, code, description FROM status_sec_med_types ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"SELECT id, code, description FROM status_sec_med_types WHERE id = $1"#;
pub const SQL_INSERT: &str = r#"INSERT INTO status_sec_med_types (code, description) VALUES ($1, $2) RETURNING id"#;
pub const SQL_UPDATE: &str = r#"UPDATE status_sec_med_types SET code = $1, description = $2 WHERE id = $3"#;
pub const SQL_DELETE: &str = "DELETE FROM status_sec_med_types WHERE id = $1";
