-- Fila de importação assíncrona de anúncios via Pulsar.
-- Criada quando POST /v1/listings/import?async=1 é chamado.
-- O consumer ImporterWorker processa e preenche listing_id + status='done'.

CREATE TABLE IF NOT EXISTS import_jobs (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    url          TEXT        NOT NULL,
    user_id      UUID,
    status       VARCHAR(20) NOT NULL DEFAULT 'queued',  -- queued | processing | done | error
    listing_id   UUID,
    error_msg    TEXT,
    queued_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_import_jobs_status ON import_jobs(status);
CREATE INDEX IF NOT EXISTS idx_import_jobs_user   ON import_jobs(user_id);
