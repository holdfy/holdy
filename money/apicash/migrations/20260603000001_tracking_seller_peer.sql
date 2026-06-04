-- Adiciona seller_peer à tabela de rastreio para o TrackingMonitor
-- notificar o vendedor em status críticos (retorno, devolução, problema, entrega).

ALTER TABLE order_tracking_status
    ADD COLUMN IF NOT EXISTS seller_peer VARCHAR(100) NOT NULL DEFAULT '';
