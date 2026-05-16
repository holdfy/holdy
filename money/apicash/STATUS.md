# APICash — estado do projeto

**Versão workspace:** 0.1.0 · **Rust:** 1.85+

## Resumo

O **APICash** é um workspace Cargo com **13 crates** de aplicação mais **`soroban-contracts`** (Wasm Soroban). A base comum é **`apicash-shared`** (`Money`, `Order`, `ApiCashError`, config, logging).

## O que está implementado

| Componente | Descrição |
|------------|-----------|
| **apicash-core** | API HTTP (Axum): pedidos, PIX, score, custódia, off-ramp; repositório **Postgres** opcional para pedidos (`APICASH_ORDERS_PG=1`); `/health`, `/ready`; auth JWT access + refresh; rate limit em login/refresh; logs/audit estruturados |
| **apicash-custody** | Escrow, yield 70/10/20; liberação **somente pelo comprador**; repositório **Postgres** opcional (`PostgresCustodyRepository`); ponte Soroban mock ou live (`--features soroban` + env) |
| **apicash-anchor** | On/off-ramp via **Anchor HTTP** (`AnchorClient`, `APICASH_STELLAR_ANCHOR_URL`) — sem rail fiat simulado |
| **apicash-antifraude** | User Score 0–1000, SEFAZ/social mock, `APPROVE` / `REVIEW` / `BLOCK` |
| **apicash-auth** | JWT HS256, middleware gateway/admin, refresh tokens |
| **apicash-admin-backend** | Dashboard, disputas, relatórios; repositórios Postgres opcionais com `APICASH_ADMIN_PG=1`; `/health`, `/ready` |
| **apicash-whatsapp** | Agente + fila + core HTTP; **transporte alvo:** `whatsapp-rust` (multi-device); webhook Cloud Meta como auxiliar; confirmação explícita de recebimento; logs com peer mascarado |
| **apicash-frontend** | UI admin Leptos (SSR) |
| **apicash-cli** | `test-flow`, `deploy-contracts`, etc. |
| **apicash-events** | Modelos e produtores Pulsar (broker opcional) |
| **soroban-contracts** | Contrato escrow Wasm para Stellar/Soroban |

## Regras de segurança (status)

- **Somente o comprador libera o dinheiro**: aplicado no `apicash-core` (JWT `sub` + `order.buyer_id`), no `apicash-custody` (`Order::is_buyer`) e no `apicash-whatsapp` (sessão amarrada ao `user_id`).
- **Auditoria/logs**: ações sensíveis emitem logs estruturados com `user_id`, `order_id`, `action`, `success` e timestamp.

## O que ainda é mock / parcial

- **Core/Admin**: pedidos, custódia, disputas e scores podem usar **Postgres** com `APICASH_ORDERS_PG=1` / `APICASH_CUSTODY_PG=1` / `APICASH_SCORES_PG=1` / `APICASH_ADMIN_PG=1`, `DATABASE_URL` e migrações (`make db-migrate`); sem flags, ficam em memória para dev/testes.
- Anchor on/off-ramp: chamadas HTTP reais ao anchor configurado (sem fallback PIX inventado no fluxo de pagamento).
- Transferência BRLx → contrato Soroban: caminho live via CLI `stellar` é best-effort; recomenda-se validar ambiente/contas na testnet antes de produção.

## Build

- Desenvolvimento: `cargo build --workspace`
- Produção otimizada: `cargo build --workspace --release` (perfil `[profile.release]` no `Cargo.toml` da raiz)

## Documentação

- [README.md](README.md) — visão geral, execução, roadmap
- [docs/arquitetura.md](docs/arquitetura.md) — diagrama, fluxos, Soroban
- [CONTRIBUTING.md](CONTRIBUTING.md) — contribuições

## Próximo passo típico

Validar **WhatsApp multi-device** com aparelho real (QR/pairing) e validar **Stellar/Soroban testnet** com credenciais reais; o caminho local mock/Postgres já está exercitado por E2E.
