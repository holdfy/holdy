-- Tabela para rastreio proativo de encomendas via WhatsApp.
-- Preenchida pelo message_handler quando o vendedor envia o código de rastreio.
-- Lida pelo TrackingMonitor a cada 30 min para detectar mudanças de status.

CREATE TABLE IF NOT EXISTS order_tracking_status (
    id                   UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id             UUID        NOT NULL,
    tracking_code        VARCHAR(50) NOT NULL,
    buyer_peer           VARCHAR(100) NOT NULL,
    last_status          VARCHAR(50) NOT NULL DEFAULT 'unknown',
    last_event_description TEXT,
    notified_at          TIMESTAMPTZ,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_order_tracking_status_unique
    ON order_tracking_status(order_id, tracking_code);

CREATE INDEX IF NOT EXISTS idx_order_tracking_status_code
    ON order_tracking_status(tracking_code);
