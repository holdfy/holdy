pub const SQL_INSERT: &str = r#"
    INSERT INTO hospital_message (payment_id, amount, retry_count, payload_json)
    VALUES ($1, $2, $3, $4)
    RETURNING id
"#;
