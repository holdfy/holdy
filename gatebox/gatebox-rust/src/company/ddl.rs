pub const SQL_LIST: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, birth_date,
           responsible_name, phone_number, email, telegram_chat_id, domanin,
           customer_status_id, is_politically_exposed_person, authentication_id, deleted_at
    FROM company ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, birth_date,
           responsible_name, phone_number, email, telegram_chat_id, domanin,
           customer_status_id, is_politically_exposed_person, authentication_id, deleted_at
    FROM company WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO company (full_name, social_name, type_person_id, document_number, birth_date,
        responsible_name, phone_number, email, telegram_chat_id, domanin,
        customer_status_id, is_politically_exposed_person, authentication_id, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE company SET full_name = $1, social_name = $2, type_person_id = $3, document_number = $4,
        birth_date = $5, responsible_name = $6, phone_number = $7, email = $8, telegram_chat_id = $9,
        domanin = $10, customer_status_id = $11, is_politically_exposed_person = $12,
        authentication_id = $13, deleted_at = $14 WHERE id = $15
"#;
pub const SQL_DELETE: &str = "DELETE FROM company WHERE id = $1";
