-- BlindPay (e outros gateways reais) audit: id do payin/payout no gateway + hash on-chain.
ALTER TABLE transaction ADD COLUMN IF NOT EXISTS gateway_tx_id VARCHAR(64);
ALTER TABLE transaction ADD COLUMN IF NOT EXISTS chain_tx_hash VARCHAR(128);

CREATE INDEX IF NOT EXISTS idx_transaction_gateway_tx_id ON transaction(gateway_tx_id);
