-- Canal de origem do pedido: whatsapp | site | app_ios | app_android.
-- Default 'site' cobre pedidos históricos (todos criados via /orders antes desta coluna existir).
ALTER TABLE orders ADD COLUMN IF NOT EXISTS platform_origin TEXT NOT NULL DEFAULT 'site';
CREATE INDEX IF NOT EXISTS idx_orders_platform_origin ON orders (platform_origin);
