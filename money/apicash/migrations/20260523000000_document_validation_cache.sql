-- Cache de CPF/CNPJ já validados para evitar re-consultas desnecessárias.
-- TTL controlado pela aplicação via coluna expires_at.

CREATE TABLE IF NOT EXISTS document_validation_cache (
    document   TEXT        NOT NULL,
    doc_type   TEXT        NOT NULL,  -- 'cpf' | 'cnpj'
    status     TEXT        NOT NULL,  -- 'valid' | 'invalid'
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (document, doc_type)
);

CREATE INDEX IF NOT EXISTS idx_document_cache_expires ON document_validation_cache (expires_at);
