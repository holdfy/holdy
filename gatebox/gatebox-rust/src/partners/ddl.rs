pub const SQL_LIST: &str = r#"
    SELECT id, partners_list_id, description, document, account, branch, authentication_id,
           client_id, client_secret, authentication, password, whpix_in_id, whpix_out_id,
           type_authorize_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           fixed_ref_cash_in, fixed_ref_cash_out, percent_ref_cashin, percent_ref_cashout, active
    FROM partners ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, partners_list_id, description, document, account, branch, authentication_id,
           client_id, client_secret, authentication, password, whpix_in_id, whpix_out_id,
           type_authorize_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           fixed_ref_cash_in, fixed_ref_cash_out, percent_ref_cashin, percent_ref_cashout, active
    FROM partners WHERE id = $1
"#;
pub const SQL_GET_BY_AUTHENTICATION_ID: &str = r#"
    SELECT id, partners_list_id, description, document, account, branch, authentication_id,
           client_id, client_secret, authentication, password, whpix_in_id, whpix_out_id,
           type_authorize_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           fixed_ref_cash_in, fixed_ref_cash_out, percent_ref_cashin, percent_ref_cashout, active
    FROM partners WHERE authentication_id = $1 AND active = true ORDER BY id LIMIT 1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO partners (partners_list_id, description, document, account, branch, authentication_id,
        client_id, client_secret, authentication, password, whpix_in_id, whpix_out_id,
        type_authorize_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
        fixed_ref_cash_in, fixed_ref_cash_out, percent_ref_cashin, percent_ref_cashout, active)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE partners SET partners_list_id = $1, description = $2, document = $3, account = $4, branch = $5,
        authentication_id = $6, client_id = $7, client_secret = $8, authentication = $9, password = $10,
        whpix_in_id = $11, whpix_out_id = $12, type_authorize_id = $13, fixed_cash_in = $14, fixed_cash_out = $15,
        percent_cashin = $16, percent_cashout = $17, fixed_ref_cash_in = $18, fixed_ref_cash_out = $19,
        percent_ref_cashin = $20, percent_ref_cashout = $21, active = $22 WHERE id = $23
"#;
pub const SQL_DELETE: &str = "DELETE FROM partners WHERE id = $1";
