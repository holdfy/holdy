pub const SQL_LIST: &str = r#"
    SELECT id, description, site, contact, active FROM partners_list ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"SELECT id, description, site, contact, active FROM partners_list WHERE id = $1"#;
pub const SQL_INSERT: &str = r#"INSERT INTO partners_list (description, site, contact, active) VALUES ($1, $2, $3, $4) RETURNING id"#;
pub const SQL_UPDATE: &str = r#"UPDATE partners_list SET description = $1, site = $2, contact = $3, active = $4 WHERE id = $5"#;
pub const SQL_DELETE: &str = "DELETE FROM partners_list WHERE id = $1";
