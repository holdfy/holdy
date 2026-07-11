pub const SQL_LIST: &str = r#"
    SELECT a.id, a.account_number, a.branch, a.account_type_id, a.account_status_id,
           a.deleted_at, a.authentication_id,
           c.type_person_id
    FROM accounts a
    LEFT JOIN customer c ON c.authentication_id = a.authentication_id AND c.deleted_at IS NULL
    ORDER BY a.id LIMIT $1 OFFSET $2
"#;

pub const SQL_GET_BY_ID: &str = r#"
    SELECT a.id, a.account_number, a.branch, a.account_type_id, a.account_status_id,
           a.deleted_at, a.authentication_id,
           c.type_person_id
    FROM accounts a
    LEFT JOIN customer c ON c.authentication_id = a.authentication_id AND c.deleted_at IS NULL
    WHERE a.id = $1
"#;

pub const SQL_GET_BY_AUTHENTICATION_ID: &str = r#"
    SELECT a.id, a.account_number, a.branch, a.account_type_id, a.account_status_id,
           a.deleted_at, a.authentication_id,
           c.type_person_id
    FROM accounts a
    LEFT JOIN customer c ON c.authentication_id = a.authentication_id AND c.deleted_at IS NULL
    WHERE a.authentication_id = $1 AND a.deleted_at IS NULL LIMIT 1
"#;

pub const SQL_INSERT: &str = r#"
    INSERT INTO accounts (account_number, branch, account_type_id, account_status_id,
                          deleted_at, authentication_id)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING id
"#;

pub const SQL_UPDATE: &str = r#"
    UPDATE accounts SET
        account_number = $1, branch = $2, account_type_id = $3,
        account_status_id = $4, deleted_at = $5, authentication_id = $6
    WHERE id = $7
"#;

pub const SQL_DELETE: &str = "DELETE FROM accounts WHERE id = $1";
