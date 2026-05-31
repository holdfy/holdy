

-- Copiado de gateboxgo/database/insert.sql (seed DEV)
-- Popula tabelas de tipos, contas de teste e saldo inicial.

-- Account Status Types
INSERT INTO account_status_types (id, code, description) VALUES (1, 'ACTIVE', 'Active');
INSERT INTO account_status_types (id, code, description) VALUES (2, 'INATIVE', 'Inactive');
INSERT INTO account_status_types (id, code, description) VALUES (3, 'DELETED', 'Deleted');
INSERT INTO account_status_types (id, code, description) VALUES (4, 'PENDING_KYC', 'Pending KYC');
INSERT INTO account_status_types (id, code, description) VALUES (5, 'REJECTED_KYC', 'Rejected KYC');

-- Account Types
INSERT INTO account_types (id, code, description) VALUES (1, 'BANKACCOUNT', 'Bank Account');
INSERT INTO account_types (id, code, description) VALUES (2, 'GRAPHICACCOUNT', 'Graphic Account');

-- Address Types
INSERT INTO address_types (id, code, description) VALUES (1, 'RESIDENTIAL', 'Residential');
INSERT INTO address_types (id, code, description) VALUES (2, 'COMMERCIAL', 'Commercial');
INSERT INTO address_types (id, code, description) VALUES (3, 'BILLING', 'Billing');

-- Customer Status Types
INSERT INTO customer_status_types (id, code, description) VALUES (1, 'PENDING_KYC', 'Pending KYC');
INSERT INTO customer_status_types (id, code, description) VALUES (2, 'REJECTED_KYC', 'Rejected KYC');
INSERT INTO customer_status_types (id, code, description) VALUES (3, 'APPROVED', 'Approved');

-- Invoice Status Types
INSERT INTO invoice_status_types (id, code, description) VALUES (1, 'CREATED', 'Created');
INSERT INTO invoice_status_types (id, code, description) VALUES (2, 'DONE', 'Done');
INSERT INTO invoice_status_types (id, code, description) VALUES (3, 'CANCEL', 'Cancelled');

-- Invoice Types
INSERT INTO invoice_types (id, code, description) VALUES (1, 'DYNAMIC', 'Dynamic');
INSERT INTO invoice_types (id, code, description) VALUES (2, 'STATIC', 'Static');
INSERT INTO invoice_types (id, code, description) VALUES (3, 'FIXED', 'Fixed');

-- PIX Key Types
INSERT INTO pix_key_types (id, code, description) VALUES (1, 'CPF', 'CPF');
INSERT INTO pix_key_types (id, code, description) VALUES (2, 'CNPJ', 'CNPJ');
INSERT INTO pix_key_types (id, code, description) VALUES (3, 'PHONE', 'Phone Number');
INSERT INTO pix_key_types (id, code, description) VALUES (4, 'EMAIL', 'Email');
INSERT INTO pix_key_types (id, code, description) VALUES (5, 'EVP', 'Random Key (EVP)');
INSERT INTO pix_key_types (id, code, description) VALUES (6, 'QRCODE', 'QR Code');

-- Type Person Types
INSERT INTO type_person_types (id, code, description) VALUES (1, 'NATURAL_PERSON', 'Natural Person');
INSERT INTO type_person_types (id, code, description) VALUES (2, 'LEGAL_PERSON', 'Legal Person');

-- Type Auth Types
INSERT INTO type_auth_types (id, code, description) VALUES (1, 'CUSTOMER', 'Customer');
INSERT INTO type_auth_types (id, code, description) VALUES (2, 'ADMIN', 'Administrator');
INSERT INTO type_auth_types (id, code, description) VALUES (3, 'MANAGER', 'Manager');

-- KYC Risk Types
INSERT INTO kyc_risk_types (id, code, description) VALUES (1, 'NORISK', 'No Risk');
INSERT INTO kyc_risk_types (id, code, description) VALUES (2, 'LOW', 'Low Risk');
INSERT INTO kyc_risk_types (id, code, description) VALUES (3, 'MEDIUM', 'Medium Risk');
INSERT INTO kyc_risk_types (id, code, description) VALUES (4, 'HARD', 'High Risk');

-- Status Controle Med Types
INSERT INTO status_controle_med_types (id, code, description) VALUES (1, 'OPEN', 'Open');
INSERT INTO status_controle_med_types (id, code, description) VALUES (2, 'ACCEPTED', 'Accepted');
INSERT INTO status_controle_med_types (id, code, description) VALUES (3, 'WAITING_RESPONSE', 'Waiting Response');
INSERT INTO status_controle_med_types (id, code, description) VALUES (4, 'RETURNED', 'Returned');
INSERT INTO status_controle_med_types (id, code, description) VALUES (5, 'CLOSED', 'Closed');

-- Status Sec Med Types
INSERT INTO status_sec_med_types (id, code, description) VALUES (1, 'OPEN', 'Open');
INSERT INTO status_sec_med_types (id, code, description) VALUES (2, 'RETURNED', 'Returned');
INSERT INTO status_sec_med_types (id, code, description) VALUES (3, 'BLOCKED', 'Blocked');
INSERT INTO status_sec_med_types (id, code, description) VALUES (4, 'ACCEPTED', 'Accepted');

-- Status Transaction Types
INSERT INTO status_transaction_types (id, code, description) VALUES (1, 'NEW', 'New');
INSERT INTO status_transaction_types (id, code, description) VALUES (2, 'QUEUED', 'Queued');
INSERT INTO status_transaction_types (id, code, description) VALUES (3, 'AWAITING', 'Awaiting');
INSERT INTO status_transaction_types (id, code, description) VALUES (4, 'COMPLETED', 'Completed');
INSERT INTO status_transaction_types (id, code, description) VALUES (5, 'ERROR', 'Error');
INSERT INTO status_transaction_types (id, code, description) VALUES (6, 'REFUNDED', 'Refunded');
INSERT INTO status_transaction_types (id, code, description) VALUES (7, 'FAILED', 'Failed');
INSERT INTO status_transaction_types (id, code, description) VALUES (8, 'DROP', 'Dropped');
INSERT INTO status_transaction_types (id, code, description) VALUES (9, 'CANCEL', 'Cancelled');
INSERT INTO status_transaction_types (id, code, description) VALUES (10, 'TESTING', 'Testing');
INSERT INTO status_transaction_types (id, code, description) VALUES (11, 'PROCESSING', 'Processing');

-- Styled Types
INSERT INTO styled_types (id, code, description) VALUES (1, 'CUSTOM', 'Custom');
INSERT INTO styled_types (id, code, description) VALUES (2, 'DEFAULT', 'Default');

-- Sub Type Transaction Types
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (1, 'PIX', 'PIX');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (2, 'DPIX', 'Direct PIX');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (3, 'P2P', 'Peer to Peer');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (4, 'BOLETO', 'Boleto');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (5, 'TTO', 'TTO');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (6, 'TPO', 'TPO');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (7, 'SMD', 'SMD');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (8, 'BLK', 'BLK');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (9, 'MEDC', 'MEDC');
INSERT INTO sub_type_transaction_types (id, code, description) VALUES (10, 'MEDD', 'MEDD');

-- Type Authorize Types
INSERT INTO type_authorize_types (id, code, description) VALUES (1, 'BASICAUTH', 'Basic Auth');
INSERT INTO type_authorize_types (id, code, description) VALUES (2, 'BEARTOKEN', 'Bearer Token');
INSERT INTO type_authorize_types (id, code, description) VALUES (3, 'HEADER', 'Header Auth');

-- Type Transaction Types
INSERT INTO type_transaction_types (id, code, description) VALUES (1, 'DEBIT', 'Debit');
INSERT INTO type_transaction_types (id, code, description) VALUES (2, 'CREDIT', 'Credit');

-- Webhook Types
INSERT INTO webhook_types (id, code, description) VALUES (1, 'PIX_PAY_IN', 'PIX Pay In');
INSERT INTO webhook_types (id, code, description) VALUES (2, 'PIX_PAY_OUT', 'PIX Pay Out');
INSERT INTO webhook_types (id, code, description) VALUES (3, 'PIX_REVERSAL', 'PIX Reversal');
INSERT INTO webhook_types (id, code, description) VALUES (4, 'PIX_REVERSAL_OUT', 'PIX Reversal Out');
INSERT INTO webhook_types (id, code, description) VALUES (5, 'PIX_REFUND', 'PIX Refund');
INSERT INTO webhook_types (id, code, description) VALUES (6, 'BILLPAYMENT', 'Bill Payment');
INSERT INTO webhook_types (id, code, description) VALUES (7, 'CREDIT_CARD_OUT', 'Credit Card Out');
INSERT INTO webhook_types (id, code, description) VALUES (8, 'CREDIT_CARD_CHARGEBACK', 'Credit Card Chargeback');

-- Type External Types
INSERT INTO type_external_types (id, code, description) VALUES (1, 'IN', 'Incoming');
INSERT INTO type_external_types (id, code, description) VALUES (2, 'OUT', 'Outgoing');

-- AUTHENTICATION (Usuários e Credenciais)
-- Senhas dev (bcrypt): admin_gateway/admin_company → admin · customer1 → customer1 · customer2 → customer2
INSERT INTO authentication (id, name, username, password, type_auth_id, active, force_reset) VALUES
  (1, 'Admin Gateway', 'admin_gateway', '$2y$10$ZZGs12LC0gZ.TulY3ZS7AeTjFc/rfQhXYvWkA/sbHdMUW0XnC/qTC', 2, true, false),
  (2, 'Admin Company', 'admin_company', '$2y$10$ZZGs12LC0gZ.TulY3ZS7AeTjFc/rfQhXYvWkA/sbHdMUW0XnC/qTC', 2, true, false),
  (3, 'Customer Test 1', 'customer1', '$2y$10$bxhdVlwL8.eCzhq1KV7cc.IU9AYbo/uvf.VwVi7kvucFero.0eo0S', 1, true, false),
  (4, 'Customer Test 2', 'customer2', '$2y$10$cdvgcjsRliRoKOgWdeEeIuh3h52Bj7//tInDkSLMMlqB8iH35Ab7.', 1, true, false)
ON CONFLICT (id) DO NOTHING;

SELECT setval('authentication_id_seq', (SELECT MAX(id) FROM authentication), true);

-- PARTNERS LIST
INSERT INTO partners_list (id, description, site, contact, active) VALUES
  (1, 'SevenTrust', 'https://seventrust.com', 'contato@seventrust.com', true),
  (2, 'Sulcred', 'https://sulcred.com', 'contato@sulcred.com', true)
ON CONFLICT (id) DO NOTHING;

SELECT setval('partners_list_id_seq', (SELECT MAX(id) FROM partners_list), true);

-- PARTNERS
INSERT INTO partners (id, partners_list_id, description, authentication_id, fixed_cash_out, percent_cashout, fixed_cash_in, percent_cashin, fixed_ref_cash_out, percent_ref_cashout, active) VALUES
  (10, 1, 'SevenTrust - Admin Gateway', 1, 0.10, 0.00, 0.00, 0.00, 0.00, 0.00, true),
  (11, 2, 'Sulcred - Admin Gateway', 1, 0.10, 0.00, 0.00, 0.00, 0.00, 0.00, true),
  (20, 1, 'SevenTrust - Admin Company', 2, 0.10, 0.00, 0.00, 0.00, 0.00, 0.00, true),
  (21, 2, 'Sulcred - Admin Company', 2, 0.10, 0.00, 0.00, 0.00, 0.00, 0.00, true)
ON CONFLICT (id) DO NOTHING;

SELECT setval('partners_id_seq', (SELECT MAX(id) FROM partners), true);

-- ACCOUNTS
INSERT INTO accounts (id, account_number, branch, account_type_id, account_status_id, authentication_id) VALUES
  (1, '1000001', '0001', 1, 1, 1),
  (2, '2000001', '0001', 1, 1, 3),
  (3, '2000002', '0001', 1, 1, 4)
ON CONFLICT (id) DO NOTHING;

SELECT setval('accounts_id_seq', (SELECT MAX(id) FROM accounts), true);

-- FEES
INSERT INTO fees (id, account_id, fixed_cash_out, percent_cashout, fixed_cash_in, percent_cashin, fixed_ref_cash_out, percent_ref_cashout) VALUES
  (1, 2, 0.10, 0.00, 0.00, 0.00, 0.00, 0.00),
  (2, 3, 0.10, 0.00, 0.00, 0.00, 0.00, 0.00)
ON CONFLICT (id) DO NOTHING;

SELECT setval('fees_id_seq', (SELECT MAX(id) FROM fees), true);

-- TRANSAÇÕES DE SALDO INICIAL
INSERT INTO transaction (
  id, account_id, type_transaction_id, status_transaction_id,
  sub_type_transaction_id, amount, key, name, document_number,
  external_id, remittance_information,
  gateway, pix_operation_type,
  requested_amount, net_amount, total_amount,
  fee_fixed, fee_percent_rate, fee_percent_amount, fee_total,
  ref_fee_fixed, ref_fee_percent_rate, ref_fee_percent_amount, ref_fee_total,
  gateway_fee, platform_fee,
  fees_calculated_at, fee_calculation_version
) VALUES
  (1000000001, 2, 2, 4, 1, 10000.00, 'SALDO_INICIAL', 'Crédito Inicial Conta 2', '12345678900',
   'INIT_BALANCE_001', 'Saldo inicial para testes de PIX',
   NULL, NULL,
   10000.00, 10000.00, 10000.00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, NOW(), 'v1.0'),
  (1000000002, 3, 2, 4, 1, 10000.00, 'SALDO_INICIAL', 'Crédito Inicial Conta 3', '98765432100',
   'INIT_BALANCE_002', 'Saldo inicial para testes de PIX',
   NULL, NULL,
   10000.00, 10000.00, 10000.00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, NOW(), 'v1.0')
ON CONFLICT (id) DO NOTHING;

