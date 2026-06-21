# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Plano de execução ativo

Existe um plano de desenvolvimento detalhado em **`PLANO_EXECUCAO.md`** na raiz do repositório.
Leia-o antes de começar qualquer trabalho. Ele contém:
- Estado real de cada componente (o que funciona, o que é stub, o que falta)
- 7 fases de execução com arquivos específicos, código e ordem de prioridade
- Decisões arquiteturais já tomadas (não mudar sem revisão)
- Checklist de progresso por fase (`[ ]` = pendente, `[x]` = concluído)

**Próximos passos imediatos:** Fase 0 (ativar Postgres) e Fase 7.1 (bug CNPJ no Gateway Next).

## Repository overview

Monorepo with four main areas:

```
holdy/
├── money/                    # Orchestration: shared .env, Docker, scripts
│   ├── apicash/              # Rust workspace — 13 crates + Soroban contracts
│   ├── docker-compose.yml    # Postgres único (:5432), Redis, Pulsar
│   ├── setup-env.sh          # Bootstrap: creates .env, symlinks, starts infra
│   ├── runinfra.sh           # Docker lifecycle + SQLx migrations
│   └── runapp.sh             # Starts/stops host processes (APICash, Gatebox, Banco)
│
├── gatebox/                  # PIX gateway stack (also reachable as money/gatebox symlink)
│   ├── gatebox-rust/         # PIX API, Rust, port 8081 (GB_API_PORT)
│   ├── simulador_rust/       # Partner simulators (Sulcred, SevenTrust)
│   ├── front-gatebox/        # Admin dashboard, React
│   ├── wallet/               # Wallet service, Go
│   └── banco/                # Simulated external bank: Flutter app + Go API
│
└── whatsapp_qrcode/          # QR pairing output dir (runtime-generated)
```

**Single source of truth for config**: `money/.env`. Both `money/apicash/.env` and `gatebox/gatebox-rust/.env` are symlinks to it. Edit only `money/.env`.

## Setup

```bash
cd money
cp .env.example .env          # first time only
./setup-env.sh                # creates symlinks and starts Docker infra
```

To skip Docker: `SKIP_INFRA=1 ./setup-env.sh`

## Common commands

All infra commands run from `money/`:

```bash
./runinfra.sh start|stop|status|migrate|logs

./runapp.sh start all            # Gatebox + APICash (set RUNAPP_AUTO_GATEBOX=1)
./runapp.sh start apicash|gatebox|banco
./runapp.sh stop|restart|status|build|logs [component]
```

APICash commands from `money/apicash/`:

```bash
make build-all                   # cargo build --workspace
make test-all                    # cargo test --workspace
make run-core                    # API pública, port 3000
make run-admin                   # Admin, port 3001
make run-frontend                # Leptos SSR, port 3002
make run-whatsapp                # WhatsApp webhook
make dev                         # core + admin + whatsapp in parallel (needs npx)
cargo test -p <crate-name>       # single crate tests
cargo run -p apicash-cli -- test-flow   # E2E flow (set APICASH_AUTH_DISABLED=1 for dev)
```

Lint and format:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

## APICash workspace architecture

13 crates under `money/apicash/crates/`:

| Crate | Role |
|-------|------|
| `apicash-shared` | Domain types (`Money`, `Order`, `ApiCashError`), config, logging — every other crate depends on this |
| `apicash-core` | Public API (Axum, port 3000): orders, PIX, score, escrow settle |
| `apicash-auth` | JWT middleware (HS256, access + refresh), rate limiting via `tower_governor` |
| `apicash-custody` | Escrow and yield; feature `soroban` enables `LiveSorobanBridge` via `stellar` CLI |
| `apicash-anchor` | Stellar/Anchor on/off-ramp; feature `soroban` links `soroban-sdk` |
| `apicash-antifraude` | User Score (0–1000), APPROVE/REVIEW/BLOCK recommendations |
| `apicash-whatsapp` | WhatsApp multi-device (`whatsapp-rust`) conversational flow |
| `apicash-admin-backend` | Admin API (port 3001) |
| `apicash-frontend` | Leptos 0.7 SSR dashboard (port 3002) |
| `apicash-events` | Apache Pulsar producers |
| `apicash-disputes` | Dispute handling |
| `apicash-notifications` | Notification dispatch |
| `apicash-cli` | Developer CLI: `test-flow`, `deploy-contracts` |
| `soroban-contracts` | Soroban Wasm escrow contract (lock/release on-chain) |

**Payment flow**: PIX IN → anchor on-ramp → BRLx token → Soroban escrow `lock` → buyer confirms delivery → `release` → off-ramp PIX.

**Fiat rail**: controlled by `APICASH_FIAT_RAIL` in `.env` (`simulated` for local dev, `anchor` for real Stellar/Anchor).

**Soroban mode**: requires `APICASH_SOROBAN_ENABLED=1` + `APICASH_SOROBAN_STRICT=1`. With `APICASH_REQUIRE_TESTNET=1`, the core rejects mock on-chain hashes (`mock_*`, `mock_stellar_*`).

**x402 protocol**: optional micropayments on Base Sepolia. Enable with `APICASH_X402_REQUIRED=1`. Routes without JWT return `402 Payment Required` with a `Payment-Required` header.

## Gatebox business rules (gatebox-rust)

Gatebox is a PIX gateway using the shared Postgres instance (database `dubai-cash` on port 5432). Key invariants:

- **Idempotency**: same `external_id` + `account_id` + `amount` + same calendar day → rejected as duplicate
- **Balance**: `SUM(CREDIT - DEBIT)` where `status_transaction_id IN (3=AWAITING, 4=COMPLETED)` minus `sec_med` blocked amounts
- **Fee hierarchy**: per-customer `fees` table overrides `partners` table defaults
- **MED** (security reserve): `percentsec_med %` of every PIX IN is locked for 90 days in `sec_med`, deducted from available balance
- **Transaction sub-types**: PIX(1), DPIX(2=reversal), TTO(3=platform fee), TPO(4=partner fee), SMD(5=MED)
- Banco (gatebox/banco) integrates with Gatebox only via its external API — no shared DB or internal code

## Stellar testnet

Public Stellar testnet (not localhost):

- Horizon: `https://horizon-testnet.stellar.org`
- Soroban RPC: `https://soroban-testnet.stellar.org`

Testnet setup scripts (from `money/apicash/`):

```bash
scripts/bootstrap-testnet-env.sh    # fund holdfy-* identities, deploy BRLx SAC, update .env
scripts/soroban-testnet-check.sh    # validate stellar CLI + required env vars
scripts/soroban-testnet-deploy.sh   # deploy escrow contract, prints APICASH_SOROBAN_ESCROW_CONTRACT_ID
scripts/testnet-onchain-smoke.sh    # transfer BRLx → escrow + lock (no full order required)
scripts/validate-x402-env.sh        # validate X402_* vars when APICASH_X402_REQUIRED=1
```

Stellar amounts use 7 decimal places (e.g. `5000000` = 0.5 BRLx).

## Coding conventions

- **Monetary values**: always use `Money` / `rust_decimal::Decimal` — never `f64` in business logic
- **Cross-crate errors**: prefer `ApiCashError` from `apicash-shared` on public surfaces
- **Observability**: `tracing` (`info!`, `warn!`, `error!`) on service paths and external failures
- **Soroban feature flag**: `apicash-custody --features soroban` for on-chain bridge; workspace compiles with automatic env-driven bridge selection

## Sensitive files (never commit)

- `money/.env` and any real-secret `.env`
- `**/github.txt` or any password notes
- `**/.runapp/` (PIDs, logs)
- `**/target/`, Flutter build artifacts (`build/`, `.dart_tool/`)
- `*.tar.gz`, `*.pem`, `*.key`


## Manter fluxos atualizados

- `site e whatsapp` toda vez que atualizar o fluxo do whatsapp deve-se atualizar o fluxo site. o site esta multilangue e os 3 idiomas devem serem atualizados tambem.

- `matenter as funcionalidades do whatsapp` jamais alerar uma funcionalidade que o whatsapp utilize do codigo para atender a fluxo web ou de apis, so fazer isso se for um pedido explicito.

