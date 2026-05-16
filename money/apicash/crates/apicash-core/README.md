# apicash-core

Main Axum REST API for APICash. It orchestrates **apicash-antifraude**, **apicash-custody**, and **apicash-anchor** behind a single gateway.

## Run

```bash
cd apicash
cargo run -p apicash-core
```

Defaults to `0.0.0.0:3000` (override with `APICASH_HTTP_BIND` or `APICASH_HTTP_PORT`).

Optional: set `APICASH_API_KEY` and send `Authorization: Bearer <key>` on protected routes. If unset, auth middleware allows all traffic (local development).

Stellar/Anchor settings use `apicash-anchor`’s `StellarConfig::from_env()` when all variables are present; otherwise sensible testnet fallbacks apply (see `APICASH_STELLAR_*` in the anchor crate docs).

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Liveness |
| POST | `/orders` | Create order (risk → custody lock → anchor PIX deposit) |
| GET | `/orders/{id}` | Order snapshot |
| POST | `/payments/pix` | PIX on-ramp with risk gate |
| POST | `/custody/release` | Release escrow after confirmation |

## Tests

```bash
cargo test -p apicash-core
```
