-- App log table for backoffice logs
CREATE TABLE IF NOT EXISTS app_log (
    id BIGSERIAL PRIMARY KEY,
    level VARCHAR(20) NOT NULL,
    service VARCHAR(100) NOT NULL DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_app_log_created_at ON app_log(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_app_log_level ON app_log(level);
CREATE INDEX IF NOT EXISTS idx_app_log_service ON app_log(service);
