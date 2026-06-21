CREATE TABLE IF NOT EXISTS proposals (
    id          UUID PRIMARY KEY,
    seller_id   UUID NOT NULL,
    buyer_id    UUID NOT NULL,
    amount      TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'pending',
    created_at  TIMESTAMPTZ NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    order_id    UUID,
    listing_id  UUID
);

CREATE INDEX IF NOT EXISTS proposals_seller_id_idx ON proposals (seller_id);
CREATE INDEX IF NOT EXISTS proposals_buyer_id_idx  ON proposals (buyer_id);
