-- Fase 3.2: Dispute management table
CREATE TABLE IF NOT EXISTS disputes (
    id                  BIGSERIAL PRIMARY KEY,
    transaction_id      BIGINT REFERENCES transaction(id) ON DELETE RESTRICT,
    account_id          BIGINT NOT NULL,
    type                VARCHAR(20) NOT NULL DEFAULT 'INFRACTION',
    status              VARCHAR(20) NOT NULL DEFAULT 'OPEN',
    reason              TEXT NOT NULL DEFAULT '',
    evidence            JSONB,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at         TIMESTAMPTZ,
    resolved_by         BIGINT,
    resolution_notes    TEXT
);

CREATE INDEX IF NOT EXISTS idx_disputes_account_id   ON disputes(account_id);
CREATE INDEX IF NOT EXISTS idx_disputes_status        ON disputes(status);
CREATE INDEX IF NOT EXISTS idx_disputes_created_at    ON disputes(created_at DESC);
