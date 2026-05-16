# apicash-shared

Central library for **APICash**: configuration structs, unified errors, monetary types (`Money` / `rust_decimal`), domain models (orders, custody, disputes, user score), constants, and small helpers (e.g. Stellar network labels).

## Modules

| Path | Purpose |
| --- | --- |
| `config` | `AppConfig` and nested sections (database, Redis, Stellar, Pulsar, WhatsApp, anti-fraud, auth, notifications) |
| `error` | `ApiCashError` for cross-crate boundaries |
| `models` | `Order`, `Payment`, `Custody`, `User`, `UserScore`, `Dispute` |
| `types` | `Money`, status enums (`OrderStatus`, `PaymentStatus`, `CustodyStatus`, `DisputeStatus`) |
| `constants` | Business defaults (custody days, yield split, score bounds) |
| `utils::stellar` | Testnet/mainnet passphrases and default public URLs |

## Configuration loading

`AppConfig::load()` reads optional `.env`, then merges variables prefixed with `APICASH__` (nested keys use `__`, e.g. `APICASH__DATABASE__URL`).

## Example

```bash
cargo run -p apicash-shared --example config_example
```

## Money

Never use `f64` for currency: use [`Money`](src/types/money.rs) everywhere.
