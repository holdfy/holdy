# apicash-custody

Escrow **custody**: lock principal per [`apicash_shared::Order`], accrue **yield** with a configurable monthly rate (applied as simple daily accrual: monthly ÷ 30 × days), and on **release** split the **yield pool** with a **70 / 10 / 20** breakdown (seller / buyer cashback / platform), aligned with [`apicash_shared::YIELD_DISTRIBUTION_PERCENT`].

## Stellar / Soroban (future)

Business rules here are intentionally pure Rust ([`rust_decimal`] / [`apicash_shared::Money`]) so they can be mirrored in **Soroban** smart contracts on Stellar and reconciled against ledger state.

Optional Cargo feature:

- **`stellar-prep`** — pulls [`soroban-sdk`](https://crates.io/crates/soroban-sdk) and [`stellar-rpc-client`](https://crates.io/crates/stellar-rpc-client) (the ecosystem’s primary Rust RPC client; sometimes referred to informally as “Stellar Rust SDK”) for future wiring. Default builds omit them to keep compile times lean.

## Main API

- [`CustodyService::lock_funds`](src/service/custody_service.rs)
- [`CustodyService::calculate_yield`](src/service/custody_service.rs) — returns [`YieldDistribution`] for the yield accrued over `days`
- [`CustodyService::release_funds`](src/service/custody_service.rs) — records total `yield_earned`, applies split, sets status to `Released`

Persistence is behind [`CustodyRepository`](src/repository/custody_repository.rs); the default [`InMemoryCustodyRepository`](src/repository/custody_repository.rs) is for tests — replace with SQLx + Postgres in production.

## Tests

```bash
cargo test -p apicash-custody
```

Enable Stellar-related crates when ready:

```bash
cargo check -p apicash-custody --features stellar-prep
```
