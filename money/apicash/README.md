# APICash

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org/)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-7A42F4.svg)](https://stellar.org/)
[![Axum](https://img.shields.io/badge/HTTP-Axum_0.8-blue.svg)](https://github.com/tokio-rs/axum)
[![Leptos](https://img.shields.io/badge/UI-Leptos_0.7-9cf.svg)](https://leptos.dev/)

## Descrição

**APICash** é uma plataforma **fintech** em **Rust** para **custódia em escrow**, **yield** com repartição configurável, integração **Stellar / Anchor** (PIX ↔ token **BRLx**), **anti-fraude** com *User Score* (0–1000), validações estilo **SEFAZ** e **redes sociais**, **disputas**, mensagens **Apache Pulsar**, canal **WhatsApp**, **JWT** (access + refresh), API **admin** e **dashboard** web (**Leptos**). Contratos **Soroban** em `soroban-contracts/` integram lock/release on-chain quando configurado.

Toda a semântica de domínio partilhada vive em **`apicash-shared`** (`Money`, `Order`, `ApiCashError`, configurações, logging).

## Tecnologias

| Área | Stack |
|------|--------|
| Linguagem | Rust **1.85+** |
| APIs HTTP | **Axum** 0.8, **Tower**, **tower_governor** (rate limit em auth) |
| Async | **Tokio** |
| Persistência | **SQLx** (Postgres); repositórios in-memory onde ainda não há DB |
| Cache / filas | **Redis** (planeado), **Apache Pulsar** |
| Auth | **jsonwebtoken** (HS256), crate **`apicash-auth`** |
| UI admin | **Leptos** 0.7 (SSR) |
| Stellar / Soroban | **stellar-rpc-client**, Horizon, **Soroban** (`soroban-contracts/`, CLI `stellar`) |
| WhatsApp | **`whatsapp-rust`** (multi-device, alinhado como alternativa Rust ao *whatsmeow*); *cloud-api* só auxiliar/legado |
| Observabilidade | **tracing**, **tracing-subscriber** (JSON com `APICASH_LOG_FORMAT=json`) |
| Infra local | **Docker Compose** (Postgres, Redis, Pulsar, pgAdmin opcional) |

## Como rodar (passo a passo)

1. **Instalar** Rust 1.85+ (`rustup update stable`), **Docker** e **Docker Compose**; opcionalmente **Node/npm** para `npx concurrently` nos targets Make, e `sqlx-cli` para migrações.

2. **Clonar / entrar no workspace**
   ```bash
   cd apicash
   ```

3. **Um único `.env` no monorepo (`money/`)** — modelo versionado: **`../.env.example`**. O `env.template` nesta pasta é **symlink** para esse ficheiro.

   Na pasta **`money/`** (um nível acima do crate):
   ```bash
   cd ..
   ./setup-env.sh
   ```
   Isto cria `money/.env`, associa **`apicash/.env`** e **`gatebox/gatebox-rust/.env`** a esse mesmo ficheiro e sobe Docker (Postgres, Redis, Pulsar…). Edite segredos apenas em **`money/.env`**. Não use **`.env.local`**.

4. **Infra / migrações** — `./setup-env.sh` já chama **`runinfra.sh`** (omitir com `SKIP_INFRA=1`). Alternativa: `./runinfra.sh start` ou, a partir daqui, `make up` / `make db-migrate`.

5. **Compilar**
   ```bash
   make build-all
   # ou
   cargo build --workspace
   ```

6. **Serviços Rust** (com a infra no ar), em terminais separados ou:
   - `make run-core` — API pública (porta **3000** por defeito, `API_PORT` / `APICASH_HTTP_PORT`)
   - `make run-admin` — admin (**3001**)
   - `make run-whatsapp` — agente WhatsApp (HTTP webhook Cloud legado + fila; transporte alvo: `whatsapp-rust`, ver crate `apicash-whatsapp`)
   - `make run-all` ou `make dev` — core + admin + WhatsApp em paralelo (requer `npx` ou script)

7. **Frontend Leptos** — `make run-frontend` (porta tipicamente **3002**, ver `Cargo.toml.leptos`).

8. **Testes e qualidade**
   ```bash
   make test-all
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   ```

9. **CLI**
   ```bash
   cargo run -p apicash-cli -- test-flow
   cargo run -p apicash-cli -- deploy-contracts
   ```

10. **Parar Docker** — `make down` ou `../runinfra.sh stop` (desde `apicash/`).

**Apps APICash (stack único):** `../runapp.sh start` na pasta `money/` (opcional por componente).

**Build de produção:** `cargo build --workspace --release` (perfil otimizado no `Cargo.toml` da raiz).

## Status atual

| Área | Estado |
|------|--------|
| API pública | Pedidos, PIX, score, custódia, off-ramp; health/ready; login + refresh com rate limit |
| Custódia + Soroban | Bridge mock ou live (`apicash-custody` com `--features soroban` + env) |
| Anchor | On/off-ramp via **Anchor HTTP** (`APICASH_STELLAR_ANCHOR_URL`) — sem rail fiat simulado no produto |
| Anti-fraude | Score, fatores, recomendação textual `APPROVE` / `REVIEW` / `BLOCK` |
| Admin + frontend | Dashboard, disputas, relatórios |
| WhatsApp | Webhook, fluxo conversacional, botões interativos (Cloud API) |
| Eventos | Tipos e produtores; broker Pulsar opcional |

Detalhe adicional: **[STATUS.md](STATUS.md)**.

## Roadmap futuro

- **PostgreSQL** — persistência real (substituir stores em memória) e migrações SQLx em CI.
- **WhatsApp Cloud API** — produção com domínio HTTPS, tokens Meta Business e políticas de opt-in.
- **KYC avançado** — integração com fornecedores de identidade e listas (PEP/sanctions) além do score atual.
- **Anchor / Stellar mainnet** — políticas de compliance e limites por utilizador.
- **Pulsar** — tópicos e consumidores em produção; *dead-letter* e idempotência.
- **Observabilidade** — métricas (OpenTelemetry), *tracing* correlacionado com `order_id`.

## Features Cargo (custódia e anchor)

| Crate | Feature | Efeito |
|-------|---------|--------|
| **apicash-custody** | `mock` | Naming symmetry / docs |
| **apicash-custody** | `soroban` | Compila `LiveSorobanBridge` (CLI `stellar`) |
| **apicash-antifraude** | `mock` | Deterministic validators without outbound HTTP (tests / CI) |
| **apicash-anchor** | `soroban` | Liga `soroban-sdk` opcional (`soroban-prep` é alias de `soroban`) |

## Ativar modo Soroban (custódia on-chain)

- **Build**: compile `apicash-custody` com feature `soroban` (o workspace já usa bridge automático por env)
- Para ligar no `apicash-core`: `cargo run -p apicash-core --no-default-features --features soroban`
- **Runtime**: defina `APICASH_SOROBAN_ENABLED=1`
- **Env mínimos (live via CLI `stellar`)**:
  - `APICASH_SOROBAN_RPC_URL`
  - `APICASH_SOROBAN_ESCROW_CONTRACT_ID`
  - `APICASH_SOROBAN_SOURCE_SECRET` (a chave que assina invocações; deve ter autoridade compatível com o contrato)
  - `APICASH_BRLX_TOKEN_CONTRACT_ID`
  - `APICASH_STELLAR_BUYER_ADDRESS` / `APICASH_STELLAR_SELLER_ADDRESS`

Fluxo: **PIX para conta do emissor/Anchor → BRLx creditado → transferência para contrato Soroban → `lock` → `confirm_delivery` → `release` → off-ramp PIX**.

### Rails fiat por env

- O `apicash-core` mantém custódia/decisão no fluxo Soroban.
- `APICASH_FIAT_RAIL=anchor` (alias legado: `sep24`): PIX ↔ token via **`AnchorClient`** contra `APICASH_STELLAR_ANCHOR_URL` (contrato documentado em `docs/anchor-http-contract.md`).
- Invariante de produto: **PIX de entrada vai para o emissor/anchor, nunca direto para o vendedor**. O repasse entre comprador e vendedor é token on-ledger + regras Soroban.

## Testando o fluxo básico

Com **`apicash-core`** na porta **3000** e auth compatível com o seu `.env` (`APICASH_AUTH_DISABLED=1` ou `APICASH_BEARER_TOKEN`):

```bash
cargo run -p apicash-cli -- test-flow
```

Fluxo: score → `POST /orders` (inicia funding e retorna QR/link) → confirmação de liquidação (`/internal/orders/settle` ou `/orders/{id}/settle`) → `custody/release` → off-ramp.

## Fluxo Principal Funcional (usuário final)

O APICash hoje suporta um fluxo de compra **informal** (P2P) com dinheiro protegido em escrow:

- **1) Criar pedido**: o comprador cria um pedido com valor e descrição (via API ou WhatsApp).
- **2) Validação anti-fraude**: calculamos score e podemos bloquear por risco.
- **3) Pagar via PIX**: o comprador paga (on-ramp PIX → BRLx).
- **4) Custódia on-chain (opcional)**: BRLx é movido para o contrato Soroban e o pedido é travado (`lock`).
- **5) Receber o produto**: após receber, o comprador confirma o recebimento.
- **6) Liberação segura**: **somente o comprador** pode liberar o pagamento; a liberação é auditada e registrada.
- **7) Saída (off-ramp)**: após release, o sistema pode executar off-ramp para PIX (mock por padrão).

## Estrutura de crates (13 + contratos)

| Crate | Função |
|-------|--------|
| `apicash-shared` | Tipos, erros, config, logging |
| `apicash-custody` | Custódia e yield |
| `apicash-anchor` | Stellar / Anchor |
| `apicash-antifraude` | Score e validações |
| `apicash-disputes` | Disputas |
| `apicash-events` | Pulsar |
| `apicash-notifications` | Notificações |
| `apicash-whatsapp` | WhatsApp |
| `apicash-auth` | JWT e middleware |
| `apicash-core` | API pública |
| `apicash-admin-backend` | API admin |
| `apicash-frontend` | UI Leptos |
| `apicash-cli` | CLI |
| `soroban-contracts` | Wasm Soroban |

Arquitetura detalhada: **[docs/arquitetura.md](docs/arquitetura.md)** · Contribuições: **[CONTRIBUTING.md](CONTRIBUTING.md)**.

## Licença

MIT (ver `[workspace.package]` no `Cargo.toml` da raiz).
