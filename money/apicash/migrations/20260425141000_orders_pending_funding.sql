-- Align orders schema with real pending funding lifecycle.
-- Funding starts first; custody/lock metadata are filled only after settlement.

ALTER TABLE orders
    ALTER COLUMN custody_id DROP NOT NULL,
    ALTER COLUMN anchor_tx_hash DROP NOT NULL;

ALTER TABLE orders
    ADD COLUMN IF NOT EXISTS fiat_rail TEXT NOT NULL DEFAULT 'simulated',
    ADD COLUMN IF NOT EXISTS gateway_in_tx_id TEXT,
    ADD COLUMN IF NOT EXISTS funding_reference TEXT,
    ADD COLUMN IF NOT EXISTS pix_br_code TEXT,
    ADD COLUMN IF NOT EXISTS sep24_interactive_url TEXT;
