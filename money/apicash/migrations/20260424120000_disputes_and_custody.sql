-- Pedidos, custódia (escrow) e disputas — alinhado com os repositórios SQLx.

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY,
    buyer_id UUID NOT NULL,
    seller_id UUID NOT NULL,
    amount NUMERIC(38, 18) NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    custody_id UUID NOT NULL,
    anchor_tx_hash TEXT NOT NULL,
    risk_score INTEGER NOT NULL,
    risk_decision TEXT NOT NULL,
    description TEXT,
    off_ramp_tx_hash TEXT,
    brlx_escrow_transfer_tx_hash TEXT,
    soroban_escrow_contract_id TEXT,
    soroban_lock_tx_hash TEXT,
    soroban_mode TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_orders_buyer_id ON orders (buyer_id);
CREATE INDEX IF NOT EXISTS idx_orders_seller_id ON orders (seller_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);

CREATE TABLE IF NOT EXISTS custody (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL UNIQUE,
    amount NUMERIC(38, 18) NOT NULL,
    status TEXT NOT NULL,
    locked_at TIMESTAMPTZ NOT NULL,
    expected_release_at TIMESTAMPTZ NOT NULL,
    actual_release_at TIMESTAMPTZ,
    yield_earned NUMERIC(38, 18),
    soroban_escrow_contract_id TEXT,
    soroban_is_mock BOOLEAN NOT NULL DEFAULT true,
    soroban_lock_tx_hash TEXT,
    soroban_release_tx_hash TEXT
);

CREATE INDEX IF NOT EXISTS idx_custody_status ON custody (status);

CREATE TABLE IF NOT EXISTS disputes (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL,
    opened_by TEXT NOT NULL,
    opened_by_user_id UUID NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL,
    evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
    opened_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    resolution_type TEXT,
    resolution_notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_disputes_order_id ON disputes (order_id);
CREATE INDEX IF NOT EXISTS idx_disputes_status ON disputes (status);

CREATE TABLE IF NOT EXISTS user_scores (
    user_id UUID PRIMARY KEY,
    score INTEGER NOT NULL,
    risk_level TEXT NOT NULL,
    factors JSONB NOT NULL DEFAULT '[]'::jsonb,
    last_updated TIMESTAMPTZ NOT NULL,
    decision TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_scores_score ON user_scores (score);
CREATE INDEX IF NOT EXISTS idx_user_scores_risk_level ON user_scores (risk_level);
