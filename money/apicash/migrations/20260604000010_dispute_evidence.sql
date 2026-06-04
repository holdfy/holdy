-- Evidências de disputa (fotos, rastreio, mensagens) com integridade SHA-256.
-- Adiciona colunas de IA e deadline na tabela disputes existente.

ALTER TABLE disputes
    ADD COLUMN IF NOT EXISTS deadline_at      TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS ai_verdict       TEXT,
    ADD COLUMN IF NOT EXISTS ai_confidence    REAL,
    ADD COLUMN IF NOT EXISTS ai_reasoning     TEXT,
    ADD COLUMN IF NOT EXISTS high_risk_buyer  BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS dispute_evidence (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    dispute_id  UUID        NOT NULL REFERENCES disputes(id) ON DELETE CASCADE,
    uploaded_by UUID        NOT NULL,
    party       TEXT        NOT NULL CHECK (party IN ('buyer', 'seller')),
    kind        TEXT        NOT NULL CHECK (kind IN ('photo', 'video', 'tracking_code', 'message', 'other')),
    minio_key   TEXT,
    content     TEXT,
    sha256      TEXT        NOT NULL,
    ai_flagged  BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dispute_evidence_dispute_id ON dispute_evidence (dispute_id);
CREATE INDEX IF NOT EXISTS idx_dispute_evidence_party      ON dispute_evidence (dispute_id, party);
