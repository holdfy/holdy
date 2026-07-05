-- Cadastro self-service com CPF/CNPJ + senha (login web, além do social login).

ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS users_document_key ON users (document) WHERE document IS NOT NULL;
