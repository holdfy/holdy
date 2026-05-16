-- Replace SEP-24 interactive URL column with neutral funding instructions + prod-aligned fiat default.

ALTER TABLE orders DROP COLUMN IF EXISTS sep24_interactive_url;

ALTER TABLE orders ADD COLUMN IF NOT EXISTS funding_instruction TEXT;

ALTER TABLE orders ALTER COLUMN fiat_rail SET DEFAULT 'anchor';
