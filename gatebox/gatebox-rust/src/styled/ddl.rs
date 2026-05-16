pub const SQL_LIST: &str = r#"
    SELECT id, url, application_name, title, primary_color, secondary_color, font_color,
           img, favicon, styled_type_id, company_id, active
    FROM styled ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, url, application_name, title, primary_color, secondary_color, font_color,
           img, favicon, styled_type_id, company_id, active
    FROM styled WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO styled (url, application_name, title, primary_color, secondary_color, font_color,
        img, favicon, styled_type_id, company_id, active)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE styled SET url = $1, application_name = $2, title = $3, primary_color = $4,
        secondary_color = $5, font_color = $6, img = $7, favicon = $8, styled_type_id = $9,
        company_id = $10, active = $11 WHERE id = $12
"#;
pub const SQL_DELETE: &str = "DELETE FROM styled WHERE id = $1";
