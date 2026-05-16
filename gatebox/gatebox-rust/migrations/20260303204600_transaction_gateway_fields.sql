-- Add gateway and pix_operation_type to transaction (for PIX IN/OUT audit)
-- Based on gateboxgo/database/alter-add-gateway-fields.sql

ALTER TABLE transaction ADD COLUMN IF NOT EXISTS gateway VARCHAR(50);
ALTER TABLE transaction ADD COLUMN IF NOT EXISTS pix_operation_type VARCHAR(20);

CREATE INDEX IF NOT EXISTS idx_transaction_gateway ON transaction(gateway);
CREATE INDEX IF NOT EXISTS idx_transaction_pix_operation_type ON transaction(pix_operation_type);
CREATE INDEX IF NOT EXISTS idx_transaction_gateway_operation ON transaction(gateway, pix_operation_type);
