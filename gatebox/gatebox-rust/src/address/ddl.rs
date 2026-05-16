pub const SQL_LIST: &str = r#"
    SELECT id, postal_code, street, number, address_complement, neighborhood, city, state,
           address_type_id, customer_id, business_id, deleted_at, company_id
    FROM address ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, postal_code, street, number, address_complement, neighborhood, city, state,
           address_type_id, customer_id, business_id, deleted_at, company_id
    FROM address WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO address (postal_code, street, number, address_complement, neighborhood, city, state,
                         address_type_id, customer_id, business_id, deleted_at, company_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE address SET postal_code = $1, street = $2, number = $3, address_complement = $4,
        neighborhood = $5, city = $6, state = $7, address_type_id = $8, customer_id = $9,
        business_id = $10, deleted_at = $11, company_id = $12 WHERE id = $13
"#;
pub const SQL_DELETE: &str = "DELETE FROM address WHERE id = $1";
