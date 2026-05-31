-- Cache de consultas NFS-e (Receita Federal) por CPF/CNPJ.
-- Evita bater no portal em toda transação para o mesmo documento.
CREATE TABLE IF NOT EXISTS nfse_document_cache (
    document      TEXT        PRIMARY KEY,  -- só dígitos (CPF 11 ou CNPJ 14)
    document_type TEXT        NOT NULL,     -- 'cpf' | 'cnpj'
    name          TEXT,
    situation     TEXT,
    cached_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
