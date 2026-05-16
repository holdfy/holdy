-- Scores antifraude persistidos para core/admin (idempotente para ambientes que já aplicaram a migração agregada).

CREATE TABLE IF NOT EXISTS user_scores (
    user_id UUID PRIMARY KEY,
    score INTEGER NOT NULL,
    risk_level TEXT NOT NULL,
    factors JSONB NOT NULL DEFAULT '[]'::jsonb,
    last_updated TIMESTAMPTZ NOT NULL,
    decision TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_user_scores_score ON user_scores (score);
CREATE INDEX IF NOT EXISTS idx_user_scores_risk_level ON user_scores (risk_level);
