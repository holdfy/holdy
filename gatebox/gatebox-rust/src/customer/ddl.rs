pub const SQL_LIST: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, birth_date,
           company_id, responsible_name, phone_number, email, customer_status_id,
           is_politically_exposed_person, authentication_id, deleted_at
    FROM customer ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, birth_date,
           company_id, responsible_name, phone_number, email, customer_status_id,
           is_politically_exposed_person, authentication_id, deleted_at
    FROM customer WHERE id = $1
"#;
pub const SQL_GET_BY_AUTHENTICATION_ID: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, birth_date,
           company_id, responsible_name, phone_number, email, customer_status_id,
           is_politically_exposed_person, authentication_id, deleted_at
    FROM customer WHERE authentication_id = $1 AND deleted_at IS NULL LIMIT 1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO customer (full_name, social_name, type_person_id, document_number, birth_date,
                          company_id, responsible_name, phone_number, email, customer_status_id,
                          is_politically_exposed_person, authentication_id, deleted_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE customer SET full_name=$1, social_name=$2, type_person_id=$3, document_number=$4,
    birth_date=$5, company_id=$6, responsible_name=$7, phone_number=$8, email=$9,
    customer_status_id=$10, is_politically_exposed_person=$11, authentication_id=$12, deleted_at=$13
    WHERE id = $14
"#;
pub const SQL_DELETE: &str = "DELETE FROM customer WHERE id = $1";

/// Search customers by email, phone, or document_number (for P2P recipient lookup).
/// Excludes deleted. Uses ILIKE for partial match. Limit 20.
pub const SQL_SEARCH: &str = r#"
    SELECT id, full_name, social_name, type_person_id, document_number, birth_date,
           company_id, responsible_name, phone_number, email, customer_status_id,
           is_politically_exposed_person, authentication_id, deleted_at
    FROM customer
    WHERE deleted_at IS NULL
      AND (
        (email ILIKE '%' || $1 || '%' AND $1 != '')
        OR (phone_number ILIKE '%' || $1 || '%' AND $1 != '')
        OR (document_number = $1 AND $1 != '')
      )
    ORDER BY id
    LIMIT 20
"#;
