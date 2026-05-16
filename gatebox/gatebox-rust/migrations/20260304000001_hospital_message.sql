-- Hospital message table for dead-letter persistence
CREATE TABLE IF NOT EXISTS hospital_message (
    id BIGSERIAL PRIMARY KEY,
    payment_id VARCHAR(100) NOT NULL DEFAULT '',
    amount DECIMAL(18,2) NOT NULL DEFAULT 0,
    retry_count INT NOT NULL DEFAULT 0,
    payload_json TEXT NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_hospital_message_created_at ON hospital_message(created_at DESC);
