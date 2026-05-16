# apicash-disputes

Serviço de **disputas** APICash: quando comprador ou vendedor contesta a entrega ou o produto, o fluxo mantém o escrow **travado** (estado `Disputed` em `apicash-custody`, espelhamento Soroban/Stellar no roadmap) até a resolução.

## API

- `DisputeService::open_dispute(order_id, opened_by, opened_by_user_id, reason, evidence)` — `opened_by` indica comprador ou vendedor; `opened_by_user_id` alimenta o evento Pulsar e auditoria.

## Fluxo

1. **`open_dispute`** — `CustodyService::mark_disputed` impede liberação automática; persiste `Dispute`; publica `DisputeOpened` via `PulsarDisputeEventSink` e `EventProducer`.
2. **`resolve_dispute`** — conforme `ResolutionType`, chama `CustodyService::release_funds` (exceto `Manual`, que só encerra a disputa no domínio).
3. **`auto_resolve_timeout`** — disputas `Open` com mais de **7 dias** (configurável em `DisputeTimeoutConfig`) são escaladas para `Manual` com nota automática.

## Postgres

Crie a tabela antes de usar `PostgresDisputeRepository`:

```sql
CREATE TABLE IF NOT EXISTS disputes (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL,
    opened_by TEXT NOT NULL,
    opened_by_user_id UUID NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL,
    evidence JSONB NOT NULL,
    opened_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    resolution_type TEXT,
    resolution_notes TEXT
);
CREATE INDEX IF NOT EXISTS idx_disputes_order_id ON disputes (order_id);
```

## Dependências de workspace

`apicash-shared`, `apicash-custody`, `apicash-events`, `sqlx`, `tokio`, `chrono`, `uuid`, `serde`, `tracing`.
