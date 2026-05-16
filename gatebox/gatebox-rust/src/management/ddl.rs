pub const SQL_LIST: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, phone_number, email,
           telegram_chat_id, customer_status_id, is_politically_exposed_person, authentication_id, deleted_at
    FROM management ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, phone_number, email,
           telegram_chat_id, customer_status_id, is_politically_exposed_person, authentication_id, deleted_at
    FROM management WHERE id = $1
"#;
pub const SQL_GET_BY_AUTHENTICATION_ID: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, phone_number, email,
           telegram_chat_id, customer_status_id, is_politically_exposed_person, authentication_id, deleted_at
    FROM management WHERE authentication_id = $1 AND deleted_at IS NULL LIMIT 1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO management (full_name, social_name, type_person_id, document_number, phone_number, email,
        telegram_chat_id, customer_status_id, is_politically_exposed_person, authentication_id, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE management SET full_name = $1, social_name = $2, type_person_id = $3, document_number = $4,
        phone_number = $5, email = $6, telegram_chat_id = $7, customer_status_id = $8,
        is_politically_exposed_person = $9, authentication_id = $10, deleted_at = $11 WHERE id = $12
"#;
pub const SQL_DELETE: &str = "DELETE FROM management WHERE id = $1";
