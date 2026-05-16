# Anchor HTTP contract (APICash ↔ `APICASH_STELLAR_ANCHOR_URL`)

APICash calls the configured anchor **HTTPS base URL** (`APICASH_STELLAR_ANCHOR_URL`) using JSON over HTTP. Paths align with common SEP-style deployments but **must match your provider’s published contract** — treat this document as the **implemented client surface**, not a fictional API.

## Required endpoints (implemented in `AnchorClient`)

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/v1/pix/deposit` | Start PIX-funded on-ramp; body includes `asset`, `amount`, `memo`. |
| `POST` | `/v1/pix/withdraw` | Request off-ramp to PIX; body includes `asset`, `amount`, `pix_key`. |
| `GET` | `/v1/pix/transaction/{transaction_id}` | Poll settlement status until terminal state. |

### `POST /v1/pix/deposit` — response fields consumed

- `transaction_id` (optional): provider reference for polling.
- `external_id` (optional): echoed correlation / memo.
- `stellar_tx_hash` (string): Stellar-side correlation hash when present.
- `status` (string): interpreted by callers (e.g. `pending`, `completed`).
- **`pix_br_code` (required for WhatsApp / PIX-first UX):** PIX “copia-e-cola” BR Code string.

If `pix_br_code` is absent or empty, **`POST /orders` in `apicash-core` returns HTTP 502** — there is no simulated fallback.

### `POST /v1/pix/withdraw` — response fields consumed

- `transaction_id`, `external_id`, `tx_hash`, `status`, `received_pix` (decimal string).

### `GET /v1/pix/transaction/{id}` — polling

JSON object must include a string `status`. Values `completed`, `settled`, or `success` are treated as settled for reconciliation purposes.

## Operational expectations

- TLS must be valid for production; anchor URL must be the **real** deployment (testnet/mainnet anchors are official infrastructure, not mocks).
- Extend or remap paths only via coordinated anchor releases — **do not** stub success without ledger settlement.
