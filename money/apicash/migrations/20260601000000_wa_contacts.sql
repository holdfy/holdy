-- Armazena dados de contato por número WhatsApp:
-- nome e CPF/CNPJ coletados durante o fluxo HoldFy.
CREATE TABLE IF NOT EXISTS wa_contacts (
    peer_key      TEXT        PRIMARY KEY,
    user_id       UUID        NOT NULL,
    name          TEXT,
    document      TEXT,
    document_type TEXT,           -- 'cpf' | 'cnpj'
    situation     TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
