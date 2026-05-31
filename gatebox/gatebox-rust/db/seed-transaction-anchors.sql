-- Dev sample rows for admin blockchain audit (idempotent on idempotency_key).
INSERT INTO transaction_anchors (
    idempotency_key, entity_type, entity_id, payload_hash,
    period_type, period_id, tx_hash, block_number, chain_id,
    anchored_at, dry_run, error_message, account_id, created_at
) VALUES
(
    'dev-pix-tx-001', 'pix_tx', '1001',
    '0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'daily', '2026-05-30',
    '0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    61234567, 80002,
    NOW() - INTERVAL '2 hours', FALSE, NULL, 1,
    NOW() - INTERVAL '2 hours'
),
(
    'dev-med-001', 'med', 'sm-42',
    '0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc',
    'daily', '2026-05-30',
    NULL, NULL, 80002,
    NULL, TRUE, 'dry_run', 1,
    NOW() - INTERVAL '1 hour'
)
ON CONFLICT (idempotency_key) DO NOTHING;
