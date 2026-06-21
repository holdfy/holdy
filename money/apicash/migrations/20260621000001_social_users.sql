-- Social login: tabela de usuários persistidos, provedores OAuth e states CSRF.

CREATE TABLE IF NOT EXISTS users (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email        TEXT UNIQUE,
    name         TEXT,
    avatar_url   TEXT,
    document     TEXT,          -- CPF (11 dígitos) ou CNPJ (14 dígitos), preenchido após vinculação
    role         TEXT NOT NULL DEFAULT 'buyer',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_social_providers (
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider     TEXT NOT NULL,   -- 'google' | 'apple' | 'facebook' | 'linkedin'
    provider_id  TEXT NOT NULL,   -- ID estável do usuário no provedor
    PRIMARY KEY (provider, provider_id)
);

-- State CSRF para o handshake OAuth (expiram em 10 min — limpeza via aplicação)
CREATE TABLE IF NOT EXISTS oauth_states (
    state         TEXT PRIMARY KEY,
    provider      TEXT NOT NULL,
    code_verifier TEXT,           -- PKCE (necessário para Apple)
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email     ON users(email);
CREATE INDEX IF NOT EXISTS idx_oauth_states_ts ON oauth_states(created_at);
