pub const SQL_LIST: &str = r#"
    SELECT id, account_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           percentsec_med, fixed_ref_cash_in, fixed_ref_cash_out, COALESCE(apagar, '') as apagar,
           percent_ref_cashin, percent_ref_cashout, deleted_at, person_type_id
    FROM fees ORDER BY id LIMIT $1 OFFSET $2
"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, account_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           percentsec_med, fixed_ref_cash_in, fixed_ref_cash_out, COALESCE(apagar, '') as apagar,
           percent_ref_cashin, percent_ref_cashout, deleted_at, person_type_id
    FROM fees WHERE id = $1
"#;
pub const SQL_GET_BY_ACCOUNT_ID: &str = r#"
    SELECT id, account_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           percentsec_med, fixed_ref_cash_in, fixed_ref_cash_out, COALESCE(apagar, '') as apagar,
           percent_ref_cashin, percent_ref_cashout, deleted_at, person_type_id
    FROM fees WHERE account_id = $1 AND deleted_at IS NULL LIMIT 1
"#;
/// Returns default fee rules for a person type (rows with no specific account attached).
/// Requires account_id = 0 as sentinel (no FK — admin inserts these manually).
pub const SQL_GET_BY_PERSON_TYPE: &str = r#"
    SELECT id, account_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
           percentsec_med, fixed_ref_cash_in, fixed_ref_cash_out, COALESCE(apagar, '') as apagar,
           percent_ref_cashin, percent_ref_cashout, deleted_at, person_type_id
    FROM fees WHERE person_type_id = $1 AND deleted_at IS NULL
    ORDER BY account_id DESC LIMIT 1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO fees (account_id, fixed_cash_in, fixed_cash_out, percent_cashin, percent_cashout,
        percentsec_med, fixed_ref_cash_in, fixed_ref_cash_out, apagar,
        percent_ref_cashin, percent_ref_cashout, deleted_at, person_type_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE fees SET account_id = $1, fixed_cash_in = $2, fixed_cash_out = $3, percent_cashin = $4,
        percent_cashout = $5, percentsec_med = $6, fixed_ref_cash_in = $7, fixed_ref_cash_out = $8,
        apagar = $9, percent_ref_cashin = $10, percent_ref_cashout = $11, deleted_at = $12,
        person_type_id = $13 WHERE id = $14
"#;
pub const SQL_DELETE: &str = "DELETE FROM fees WHERE id = $1";
