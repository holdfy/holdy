pub const SQL_LIST: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
           external_id, name, email, document_number, description, phone, amount, isbp,
           bank_name, branch, account, endtoend_id, pix_key_type_id, key, type_transaction_id,
           sub_type_transaction_id, remittance_information, status_transaction_id, msg_error,
           telegram_notification, try_count, deleted_at, endtoend_id_temp,
           gateway_tx_id, chain_tx_hash
    FROM transaction ORDER BY id LIMIT $1 OFFSET $2
"#;

/// List transactions by account_id (for admin customer extract).
pub const SQL_LIST_BY_ACCOUNT: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
           external_id, name, email, document_number, description, phone, amount, isbp,
           bank_name, branch, account, endtoend_id, pix_key_type_id, key, type_transaction_id,
           sub_type_transaction_id, remittance_information, status_transaction_id, msg_error,
           telegram_notification, try_count, deleted_at, endtoend_id_temp,
           gateway_tx_id, chain_tx_hash
    FROM transaction WHERE account_id = $1 ORDER BY id DESC LIMIT $2 OFFSET $3
"#;
pub const SQL_COUNT_BY_ACCOUNT: &str = r#"SELECT COUNT(*)::bigint FROM transaction WHERE account_id = $1"#;
pub const SQL_GET_BY_ID: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
           external_id, name, email, document_number, description, phone, amount, isbp,
           bank_name, branch, account, endtoend_id, pix_key_type_id, key, type_transaction_id,
           sub_type_transaction_id, remittance_information, status_transaction_id, msg_error,
           telegram_notification, try_count, deleted_at, endtoend_id_temp,
           gateway_tx_id, chain_tx_hash
    FROM transaction WHERE id = $1
"#;
pub const SQL_INSERT: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, email, document_number, description, phone, amount, isbp, bank_name, branch,
        account, endtoend_id, pix_key_type_id, key, type_transaction_id, sub_type_transaction_id,
        remittance_information, status_transaction_id, msg_error, telegram_notification, try_count, deleted_at, endtoend_id_temp)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29) RETURNING id
"#;
pub const SQL_UPDATE: &str = r#"
    UPDATE transaction SET account_id = $1, invoice_id = $2, partners_id = $3, transaction_id = $4,
        charger_back_id = $5, parent_id = $6, external_id = $7, name = $8, email = $9, document_number = $10,
        description = $11, phone = $12, amount = $13, isbp = $14, bank_name = $15, branch = $16, account = $17,
        endtoend_id = $18, pix_key_type_id = $19, key = $20, type_transaction_id = $21, sub_type_transaction_id = $22,
        remittance_information = $23, status_transaction_id = $24, msg_error = $25, telegram_notification = $26,
        try_count = $27, deleted_at = $28, endtoend_id_temp = $29 WHERE id = $30
"#;
pub const SQL_DELETE: &str = "DELETE FROM transaction WHERE id = $1";

/// Update PIX transaction status (used by PaymentMessageHandler).
/// gateway column added via alter-add-gateway-fields.sql
pub const SQL_UPDATE_PIX_STATUS: &str = r#"
    UPDATE transaction
    SET status_transaction_id = $1, msg_error = $2, gateway = $3
    WHERE id = $4
"#;

/// Update PIX status and endtoend_id when gateway returns (for reversal lookup).
pub const SQL_UPDATE_PIX_STATUS_ENDTOEND: &str = r#"
    UPDATE transaction
    SET status_transaction_id = $1, msg_error = $2, gateway = $3, endtoend_id = $4
    WHERE id = $5
"#;

/// Get profit: sum of TTO+TPO CREDIT to admin account (account_id=1, type=2, sub_type 5=TTO 6=TPO, status 3,4).
pub const SQL_PROFIT: &str = r#"
    SELECT COALESCE(SUM(amount), 0) as profit
    FROM transaction
    WHERE account_id = $1 AND type_transaction_id = 2
    AND sub_type_transaction_id IN (5, 6) AND status_transaction_id IN (3, 4)
"#;

/// Customer activities: customers with tx_count and last_activity (from transaction).
pub const SQL_CUSTOMER_ACTIVITIES: &str = r#"
    SELECT c.id as customer_id, c.full_name,
           COALESCE(COUNT(t.id), 0)::bigint as tx_count,
           MAX(t.created_at) as last_activity
    FROM customer c
    JOIN accounts a ON a.authentication_id = c.authentication_id AND a.deleted_at IS NULL
    LEFT JOIN transaction t ON t.account_id = a.id AND t.status_transaction_id IN (3, 4)
    WHERE c.deleted_at IS NULL
    GROUP BY c.id, c.full_name
    ORDER BY last_activity DESC NULLS LAST
    LIMIT $1 OFFSET $2
"#;
pub const SQL_CUSTOMER_ACTIVITIES_COUNT: &str = r#"
    SELECT COUNT(*)::bigint FROM (
        SELECT c.id FROM customer c
        JOIN accounts a ON a.authentication_id = c.authentication_id AND a.deleted_at IS NULL
        WHERE c.deleted_at IS NULL
    ) x
"#;

/// Get balance for account (sum of CREDIT - DEBIT for status 3,4).
pub const SQL_BALANCE: &str = r#"
    SELECT COALESCE(SUM(
        CASE WHEN type_transaction_id = 2 THEN amount
             WHEN type_transaction_id = 1 THEN -amount
             ELSE 0 END
    ), 0) as balance
    FROM transaction
    WHERE account_id = $1 AND status_transaction_id IN (3, 4)
"#;

/// Check idempotency: existing completed tx with same external_id, account, amount today.
pub const SQL_IDEMPOTENCY_CHECK: &str = r#"
    SELECT id FROM transaction
    WHERE external_id = $1 AND account_id = $2 AND status_transaction_id = 4
    AND amount = $3 AND created_at >= $4 AND created_at < $5
    LIMIT 1
"#;

/// PIX IN idempotency: existing CREDIT tx with same external_id or idempotency key.
pub const SQL_PIX_IN_IDEMPOTENCY: &str = r#"
    SELECT id FROM transaction
    WHERE (external_id = $1 OR external_id = $2)
    AND type_transaction_id = 2 AND status_transaction_id = 4
    LIMIT 1
"#;

/// Find transaction by internal id (from internalTransactionId).
#[allow(dead_code)]
pub const SQL_FIND_BY_ID: &str = r#"SELECT id FROM transaction WHERE id = $1 LIMIT 1"#;
/// Find transaction by endToEndId (external_id).
pub const SQL_FIND_BY_EXTERNAL_ID: &str = r#"
    SELECT id FROM transaction WHERE external_id = $1 ORDER BY created_at DESC LIMIT 1
"#;

/// Find original PIX OUT transaction for reversal (by endtoend_id, type=DEBIT, sub_type=PIX).
pub const SQL_FIND_ORIGINAL_FOR_REVERSAL: &str = r#"
    SELECT id, account_id, amount FROM transaction
    WHERE endtoend_id = $1 AND type_transaction_id = 1 AND sub_type_transaction_id = 1
    LIMIT 1
"#;

/// Check reversal idempotency: existing tx with same external_id and account.
pub const SQL_REVERSAL_IDEMPOTENCY: &str = r#"
    SELECT id FROM transaction
    WHERE external_id = $1 AND account_id = $2
    LIMIT 1
"#;

/// Insert TTO (sub_type=5): DEBIT or CREDIT for operational fee.
/// Params: account_id, parent_id, endtoend_id, name, document_number, amount, type_transaction_id, description
pub const SQL_INSERT_TTO: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, document_number, amount, type_transaction_id, sub_type_transaction_id,
        status_transaction_id, remittance_information)
    VALUES ($1, 0, 0, '', '', $2, $3, $4, $5, $6, $7, 5, 4, $8)
    RETURNING id
"#;

/// Insert TPO (sub_type=6): DEBIT for partner rate.
/// Params: account_id, partners_id, parent_id, endtoend_id, name, amount, description
pub const SQL_INSERT_TPO: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, amount, type_transaction_id, sub_type_transaction_id,
        status_transaction_id, remittance_information)
    VALUES ($1, 0, $2, '', '', $3, $4, $5, $6, 1, 6, 4, $7)
    RETURNING id
"#;

/// Insert SMD (sub_type=7): MED security debit.
/// Params: account_id, parent_id, endtoend_id, name, document_number, amount, description
pub const SQL_INSERT_SMD: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, document_number, amount, type_transaction_id, sub_type_transaction_id,
        status_transaction_id, remittance_information)
    VALUES ($1, 0, 0, '', '', $2, $3, $4, $5, $6, 1, 7, 4, $7)
    RETURNING id
"#;

/// Insert P2P debit (type=1 DEBIT, sub_type=3 P2P). Params: account_id, parent_id, external_id, name, document_number, amount, description
pub const SQL_INSERT_P2P_DEBIT: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, document_number, amount, type_transaction_id, sub_type_transaction_id,
        status_transaction_id, remittance_information)
    VALUES ($1, 0, 0, '', '', $2, $3, $4, $5, $6, 1, 3, 4, $7)
    RETURNING id
"#;

/// Insert P2P credit (type=2 CREDIT, sub_type=3 P2P). Params: account_id, parent_id, external_id, name, document_number, amount, description
pub const SQL_INSERT_P2P_CREDIT: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, document_number, amount, type_transaction_id, sub_type_transaction_id,
        status_transaction_id, remittance_information)
    VALUES ($1, 0, 0, '', '', $2, $3, $4, $5, $6, 2, 3, 4, $7)
    RETURNING id
"#;

/// List P2P transactions (sub_type=3) for account.
pub const SQL_LIST_P2P_BY_ACCOUNT: &str = r#"
    SELECT id, account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
           external_id, name, email, document_number, description, phone, amount, isbp,
           bank_name, branch, account, endtoend_id, pix_key_type_id, key, type_transaction_id,
           sub_type_transaction_id, remittance_information, status_transaction_id, msg_error,
           telegram_notification, try_count, deleted_at, endtoend_id_temp,
           gateway_tx_id, chain_tx_hash
    FROM transaction
    WHERE account_id = $1 AND sub_type_transaction_id = 3
    ORDER BY id DESC
    LIMIT $2 OFFSET $3
"#;

/// Insert PIX IN credit (type=2 CREDIT, sub_type=1 PIX) with gateway, pix_operation_type and fee audit fields.
/// Params: account_id, invoice_id, partners_id, external_id, name, document_number, amount, key,
///         remittance_information, gateway, pix_operation_type,
///         requested_amount, net_amount, total_amount, fee_fixed, fee_percent_rate, fee_percent_amount, fee_total,
///         partner_fixed_cash_in, partner_percent_cashin, gateway_tx_id
pub const SQL_INSERT_PIX_IN_CREDIT: &str = r#"
    INSERT INTO transaction (account_id, invoice_id, partners_id, transaction_id, charger_back_id, parent_id,
        external_id, name, document_number, description, phone, amount, isbp, bank_name, branch, account,
        endtoend_id, pix_key_type_id, key, type_transaction_id, sub_type_transaction_id,
        remittance_information, status_transaction_id, msg_error, telegram_notification, try_count,
        gateway, pix_operation_type,
        requested_amount, net_amount, total_amount, fee_fixed, fee_percent_rate, fee_percent_amount, fee_total,
        partner_fixed_cash_in, partner_percent_cashin, fees_calculated_at, fee_calculation_version, gateway_tx_id)
    VALUES ($1, $2, $3, '', '', 0, $4, $5, $6, $9, '', $7, '', '', '', '', '', 0, $8, 2, 1, $9, 4, '', false, 0, $10, $11,
        $12, $13, $14, $15, $16, $17, $18, $19, $20, NOW(), 'v1.0', $21)
    RETURNING id
"#;

/// Update chain_tx_hash for a transaction found by gateway_tx_id (payin/payout id at the gateway).
pub const SQL_UPDATE_CHAIN_TX_HASH_BY_GATEWAY_TX_ID: &str = r#"
    UPDATE transaction SET chain_tx_hash = $1 WHERE gateway_tx_id = $2
"#;

/// HoldFy transactions: PIX IN originados pelo APICash (description/remittance contém "HoldFy" ou external_id contém "order").
/// Inclui created_at para mostrar data/hora. Params: limit $1, offset $2.
pub const SQL_LIST_HOLDFY: &str = r#"
    SELECT id, name, document_number, amount, description, remittance_information,
           external_id, status_transaction_id, gateway, created_at,
           gateway_tx_id, chain_tx_hash
    FROM transaction
    WHERE deleted_at IS NULL
      AND (
        remittance_information ILIKE '%holdfy%'
        OR description ILIKE '%holdfy%'
        OR external_id ILIKE 'order%'
        OR external_id ILIKE '%order_%'
      )
    ORDER BY created_at DESC
    LIMIT $1 OFFSET $2
"#;

pub const SQL_COUNT_HOLDFY: &str = r#"
    SELECT COUNT(*)::bigint FROM transaction
    WHERE deleted_at IS NULL
      AND (
        remittance_information ILIKE '%holdfy%'
        OR description ILIKE '%holdfy%'
        OR external_id ILIKE 'order%'
        OR external_id ILIKE '%order_%'
      )
"#;
