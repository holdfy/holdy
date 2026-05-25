-- Tabela de anúncios importados via scraping (API, WhatsApp, site).
-- Plataformas: Mercado Livre, OLX, Instagram, Facebook Marketplace, TikTok Shop,
--              WhatsApp Business, Shopee, e-commerce genérico.
CREATE TABLE IF NOT EXISTS listings (
    id                UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID,
    order_id          UUID,
    source_url        TEXT,
    source_platform   VARCHAR(30),
    extractor_used    VARCHAR(50),
    title             TEXT        NOT NULL,
    description       TEXT,
    price_suggested   NUMERIC(14, 2),
    guarantee         TEXT,
    condition         VARCHAR(20),   -- 'new' | 'used' | 'refurbished'
    location          TEXT,
    seller_name       TEXT,
    seller_rating     TEXT,
    photos            JSONB       NOT NULL DEFAULT '[]',
    raw_attributes    JSONB       NOT NULL DEFAULT '{}',
    imported_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_listings_user_id        ON listings(user_id);
CREATE INDEX IF NOT EXISTS idx_listings_order_id       ON listings(order_id);
CREATE INDEX IF NOT EXISTS idx_listings_source_platform ON listings(source_platform);
CREATE INDEX IF NOT EXISTS idx_listings_imported_at    ON listings(imported_at DESC);
