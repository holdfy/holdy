-- Blockchain audit: anchored PIX / MED events (read by GET /api/v1/anchor/audit)
CREATE TABLE IF NOT EXISTS transaction_anchors (
    id               BIGSERIAL PRIMARY KEY,
    idempotency_key  VARCHAR(255) NOT NULL,
    entity_type      VARCHAR(64) NOT NULL,
    entity_id        VARCHAR(255) NOT NULL,
    payload_hash     VARCHAR(66) NOT NULL,
    period_type      VARCHAR(32),
    period_id        VARCHAR(64),
    tx_hash          VARCHAR(66),
    block_number     BIGINT,
    chain_id         BIGINT,
    anchored_at      TIMESTAMPTZ,
    dry_run          BOOLEAN NOT NULL DEFAULT FALSE,
    error_message    TEXT,
    account_id       BIGINT NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_transaction_anchors_idempotency
    ON transaction_anchors(idempotency_key);

CREATE INDEX IF NOT EXISTS idx_transaction_anchors_created_at
    ON transaction_anchors(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_transaction_anchors_entity_type
    ON transaction_anchors(entity_type);

CREATE INDEX IF NOT EXISTS idx_transaction_anchors_period
    ON transaction_anchors(period_type, period_id);
