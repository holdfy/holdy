# apicash-anchor

Stellar integration for APICash: **on-ramp** (PIX → tokenized **BRLx**), **off-ramp** (**BRLx** → PIX), **Horizon** reads, and Soroban hooks. Fiat settles via the anchor’s banking rails; the ledger holds **BRLx** (or [`StellarConfig::asset_code`](src/config/stellar_config.rs)).

## Configuration (environment only)

Set variables before calling [`StellarConfig::from_env`](src/config/stellar_config.rs) (optionally after `dotenvy::dotenv()`):

| Variable | Example |
| --- | --- |
| `APICASH_STELLAR_NETWORK` | `testnet` or `mainnet` |
| `APICASH_STELLAR_ANCHOR_URL` | `https://your-anchor.example` |
| `APICASH_STELLAR_ASSET_CODE` | `BRLx` |
| `APICASH_STELLAR_HORIZON_URL` | `https://horizon-testnet.stellar.org` |
| `APICASH_STELLAR_SECRET_KEY` | Stellar secret (KMS in production) |
| `APICASH_FIAT_RAIL` | `anchor` (legacy alias: `sep24`) |

**Never** commit real secrets.

## Crates (Rust ecosystem)

- **stellar-rpc-client** and **stellar-xdr** — re-exported for wiring.
- **soroban-sdk** — optional via `--features soroban` for Soroban escrow flows (`soroban-prep` is an alias).

## Features

| Feature | Purpose |
| --- | --- |
| `soroban` | Links `soroban-sdk` for contract integration work. |
| `soroban-prep` | Same as `soroban` (backward-compatible alias). |

## Tests

Integration-style tests use **wiremock** against `AnchorClient` (no real anchor required):

```bash
cargo test -p apicash-anchor
```

## Layout

- [`client::AnchorClient`](src/client/anchor_client.rs) — Anchor HTTP (`/v1/pix/*`); adjust paths only per your anchor contract (see workspace `docs/anchor-http-contract.md`).
- [`client::StellarClient`](src/client/stellar_client.rs) — Horizon `GET /transactions/{id}`.
