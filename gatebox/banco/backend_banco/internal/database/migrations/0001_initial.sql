CREATE TABLE IF NOT EXISTS banco_users (
  id TEXT PRIMARY KEY,
  full_name TEXT NOT NULL,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS banco_bank_accounts (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  agency TEXT NOT NULL,
  account_number TEXT NOT NULL,
  pix_key TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(user_id) REFERENCES banco_users(id)
);

CREATE TABLE IF NOT EXISTS banco_balances (
  account_id TEXT PRIMARY KEY,
  available_cents INTEGER NOT NULL DEFAULT 0,
  blocked_cents INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(account_id) REFERENCES banco_bank_accounts(id)
);

CREATE TABLE IF NOT EXISTS banco_transactions (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  type TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  status TEXT NOT NULL,
  payer TEXT,
  receiver TEXT,
  gatebox_charge_id TEXT,
  payment_id TEXT,
  details TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(account_id) REFERENCES banco_bank_accounts(id)
);

CREATE TABLE IF NOT EXISTS banco_payments (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  gatebox_charge_id TEXT NOT NULL,
  payment_method TEXT NOT NULL,
  payment_state TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  idempotency_key TEXT NOT NULL,
  external_reference TEXT,
  fail_reason TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(account_id, idempotency_key),
  FOREIGN KEY(account_id) REFERENCES banco_bank_accounts(id)
);

CREATE TABLE IF NOT EXISTS banco_payment_attempts (
  id TEXT PRIMARY KEY,
  payment_id TEXT NOT NULL,
  attempt_no INTEGER NOT NULL,
  request_payload TEXT,
  response_payload TEXT,
  status_code INTEGER,
  created_at TEXT NOT NULL,
  FOREIGN KEY(payment_id) REFERENCES banco_payments(id)
);

CREATE TABLE IF NOT EXISTS banco_simulation_settings (
  account_id TEXT PRIMARY KEY,
  auto_approve INTEGER NOT NULL DEFAULT 0,
  auto_reject INTEGER NOT NULL DEFAULT 0,
  processing_delay_ms INTEGER NOT NULL DEFAULT 0,
  random_failure_rate REAL NOT NULL DEFAULT 0,
  timeout_enabled INTEGER NOT NULL DEFAULT 0,
  insufficient_balance_enabled INTEGER NOT NULL DEFAULT 0,
  webhook_active INTEGER NOT NULL DEFAULT 1,
  gatebox_environment TEXT NOT NULL DEFAULT 'sandbox',
  updated_at TEXT NOT NULL,
  FOREIGN KEY(account_id) REFERENCES banco_bank_accounts(id)
);

CREATE TABLE IF NOT EXISTS banco_audit_logs (
  id TEXT PRIMARY KEY,
  account_id TEXT,
  action TEXT NOT NULL,
  actor TEXT NOT NULL,
  metadata TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS banco_webhook_events (
  id TEXT PRIMARY KEY,
  payment_id TEXT NOT NULL,
  target_url TEXT NOT NULL,
  payload TEXT NOT NULL,
  status TEXT NOT NULL,
  response_code INTEGER,
  response_body TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS banco_idempotency_keys (
  id TEXT PRIMARY KEY,
  operation TEXT NOT NULL,
  key_value TEXT NOT NULL UNIQUE,
  resource_id TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS banco_gatebox_integration_logs (
  id TEXT PRIMARY KEY,
  payment_id TEXT,
  operation TEXT NOT NULL,
  request_body TEXT,
  response_body TEXT,
  status_code INTEGER,
  duration_ms INTEGER,
  created_at TEXT NOT NULL
);

