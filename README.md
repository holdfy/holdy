# pos-nearx

Monorepo da stack **HoldFy / NearX**: pagamentos PIX (Gatebox), plataforma fintech **APICash** (Rust + Stellar/Soroban), app banco (Flutter), wallet Go e utilitários de pareamento WhatsApp.

Um único repositório Git na raiz; configuração partilhada em `money/.env`.

## Mapa do repositório

```
pos-nearx/
├── money/                    # Orquestração: Docker, .env, runapp / runinfra
│   ├── apicash/              # Workspace Rust (core, admin, frontend Leptos, WhatsApp, Soroban)
│   ├── docker-compose.yml    # Postgres APICash, Redis, Pulsar, Postgres Gatebox
│   ├── setup-env.sh          # Bootstrap: .env + symlinks + infra
│   ├── runinfra.sh           # Docker + migrações SQLx
│   └── runapp.sh             # Apps em host (APICash, Gatebox, backend banco)
│
├── gatebox/
│   ├── gatebox-rust/         # API PIX / gateways (Rust) — porta 8081
│   ├── simulador_rust/       # Simuladores de parceiros (Sulcred, SevenTrust, …)
│   ├── front-gatebox/        # Dashboard React
│   ├── wallet/               # Serviço wallet (Go)
│   └── banco/
│       ├── app_banco/        # App Flutter (cliente)
│       └── backend_banco/    # API Go do banco
│
└── whatsapp_qrcode/          # QR + pair.html para parear WhatsApp (gerado em runtime)
```

| Projeto | Stack | README |
|---------|--------|--------|
| APICash | Rust 1.85+, Axum, Leptos, SQLx, Pulsar | [money/apicash/README.md](money/apicash/README.md) |
| Gatebox Rust | Rust, PIX EMV | [gatebox/gatebox-rust/README.md](gatebox/gatebox-rust/README.md) |
| Simulador | Rust | [gatebox/simulador_rust/README.md](gatebox/simulador_rust/README.md) |
| Front Gatebox | React | [gatebox/front-gatebox/README.md](gatebox/front-gatebox/README.md) |
| Banco | Flutter + Go | [gatebox/banco/README.md](gatebox/banco/README.md) |

## Pré-requisitos

- **Rust** 1.85+ (`rustup update stable`)
- **Docker** e **Docker Compose**
- **Stellar CLI** (`stellar`) — testnet, deploy Soroban e transações on-chain
- **Go** 1.22+ (wallet, `backend_banco`)
- **Flutter** (opcional, `gatebox/banco/app_banco`)
- **Node/npm** (opcional, front Gatebox e `make dev` do APICash)
- `protoc` se compilar Gatebox com feature Pulsar
- `jq` (opcional) para formatar respostas JSON da API

## Primeira configuração

```bash
cd money
cp .env.example .env    # se ainda não existir
./setup-env.sh          # symlinks apicash/.env e gatebox/gatebox-rust/.env → money/.env
```

Edite **apenas** `money/.env` (segredos, `MONEY_LAN_HOST`, rails Stellar, Gatebox, WhatsApp).

`setup-env.sh` cria symlinks e, por defeito, sobe a infra Docker (`runinfra.sh`). Para só preparar ficheiros:

```bash
SKIP_INFRA=1 ./setup-env.sh
```

## Comandos do dia a dia

Todos a partir de `money/`:

### Infraestrutura (Docker)

```bash
./runinfra.sh start      # Postgres, Redis, Pulsar + migrações
./runinfra.sh stop
./runinfra.sh status
./runinfra.sh migrate    # só migrações
./runinfra.sh logs
```

Serviços típicos (ver `docker-compose.yml` e `.env`):

| Serviço | Porta default |
|---------|----------------|
| Postgres APICash | 5432 |
| Postgres Gatebox | 5433 |
| Redis | 6379 |
| Pulsar broker | 6650 |
| Pulsar admin | 8080 |

### Aplicações (host)

```bash
./runapp.sh start all       # Gatebox (se RUNAPP_AUTO_GATEBOX=1) + APICash
./runapp.sh start apicash   # core, admin, frontend, whatsapp
./runapp.sh start gatebox   # gatebox-rust
./runapp.sh start banco     # backend_banco Go
./runapp.sh stop [apicash|gatebox|banco|all]
./runapp.sh restart […]
./runapp.sh status
./runapp.sh build [apicash|banco|all]
./runapp.sh logs [apicash|gatebox|banco|all]
```

Portas HTTP habituais (sobrescrevíveis no `.env`):

| App | Porta |
|-----|-------|
| apicash-core | 3000 |
| apicash-admin | 3001 |
| apicash-frontend (Leptos) | 3002 |
| apicash-whatsapp webhook | 3010 |
| gatebox-rust | 8081 (`GB_API_PORT`) |

### APICash (dentro do crate)

```bash
cd money/apicash
make build-all
make run-core      # ou run-admin, run-frontend, run-all
make test-all
cargo run -p apicash-cli -- test-flow
```

### WhatsApp — pareamento

O `runapp.sh` grava o QR em `whatsapp_qrcode/` (variável `WA_QR_DIR` no `.env`). Abra no browser:

`whatsapp_qrcode/pair.html`

## Fluxo integrado (PIX via Gatebox)

1. `./runinfra.sh start`
2. `./runapp.sh start all` — sobe **gatebox-rust** antes do APICash quando `RUNAPP_AUTO_GATEBOX=1`
3. No `.env`: `GATEBOX_BASE_URL`, `APICASH_FIAT_RAIL=anchor` (evitar rail `simulated` em produção de dev integrado)

Sem Gatebox ativo e com `GATEBOX_BASE_URL` definido, a criação de instruções PIX pode falhar.

## Stellar testnet

A rede configurada por defeito é a **Stellar testnet** (`APICASH_STELLAR_NETWORK=testnet`, Horizon e Soroban RPC de testnet em `money/.env.example`).

| O que aparece on-chain | O que não aparece |
|------------------------|-------------------|
| Transferência BRLx (token Soroban) | PIX / Gatebox (sandbox off-chain) |
| `lock` / `release` no contrato escrow | Hashes `mock_*` ou `mock_stellar_*` |
| Deploy do contrato escrow | |

Com `APICASH_REQUIRE_TESTNET=1` (defeito no `.env.example`), o **apicash-core** exige Soroban live (`APICASH_SOROBAN_ENABLED=1`, `APICASH_SOROBAN_STRICT=1`) e recusa mocks on-chain no settle.

### Configurar testnet (primeira vez)

```bash
cd money
cp .env.example .env          # se ainda não existir
./setup-env.sh                # symlinks + infra Docker

cd apicash
scripts/bootstrap-testnet-env.sh   # chaves holdfy-* fundadas + BRLx SAC no .env
scripts/soroban-testnet-check.sh     # valida stellar CLI + variáveis
scripts/soroban-testnet-deploy.sh    # build Wasm + deploy escrow (grava APICASH_SOROBAN_ESCROW_CONTRACT_ID)

cd ..
./runapp.sh build apicash
./runapp.sh start apicash       # ou só core: ver nota abaixo
```

Variáveis críticas em `money/.env` (não commitar):

| Variável | Descrição |
|----------|-----------|
| `APICASH_REQUIRE_TESTNET` | `1` = obriga tx reais na testnet |
| `APICASH_SOROBAN_ENABLED` | `1` = bridge Soroban live |
| `APICASH_SOROBAN_ESCROW_CONTRACT_ID` | Contrato escrow (`C…`) |
| `APICASH_BRLX_TOKEN_CONTRACT_ID` | Token BRLx SAC (`C…`) |
| `APICASH_SOROBAN_SOURCE_SECRET` | Conta deployer (`S…`) |
| `APICASH_SOROBAN_BUYER_SOURCE` | Conta comprador para assinar |
| `APICASH_STELLAR_BUYER_ADDRESS` | `G…` comprador |
| `APICASH_STELLAR_SELLER_ADDRESS` | `G…` vendedor |
| `GATEBOX_BASE_URL` | Ex.: `http://127.0.0.1:8081` se usar `./runapp.sh start gatebox` |

Smoke on-chain sem pedido completo (transfer + lock):

```bash
cd money/apicash
scripts/testnet-onchain-smoke.sh
```

Fluxo E2E com pedido (`apicash-cli`): requer Gatebox acessível em `GATEBOX_BASE_URL` e auth configurada (`APICASH_AUTH_DISABLED=1` ou credenciais de teste).

```bash
cd money/apicash
export APICASH_AUTH_DISABLED=1   # só dev local
cargo run -p apicash-cli -- test-flow
```

### API — últimas transações testnet

Endpoint público (sem JWT), disponível com **apicash-core** na porta **3000**:

```http
GET /testnet/transactions?limit=10&source=all
```

**Parâmetros de query**

| Parâmetro | Defeito | Descrição |
|-----------|---------|-----------|
| `limit` | `10` | Máximo de itens (1–50) |
| `source` | `all` | `db` — só hashes guardados nos pedidos; `horizon` — contas do `.env` via Horizon; `all` — união dos dois |

**Exemplo**

```bash
curl -s 'http://127.0.0.1:3000/testnet/transactions?limit=10' | jq
curl -s 'http://127.0.0.1:3000/testnet/transactions?source=horizon&limit=5' | jq
curl -s 'http://127.0.0.1:3000/testnet/transactions?source=db&limit=10' | jq
```

**Resposta (campos principais)**

```json
{
  "network": "testnet",
  "horizon_url": "https://horizon-testnet.stellar.org",
  "limit": 10,
  "count": 2,
  "transactions": [
    {
      "hash": "9b9fb616b1bfe9e521d342d7ea13fb7f881c4385b1e0dd32ad990fae66ab6c36",
      "kind": "horizon_buyer",
      "order_id": "00000000-0000-0000-0000-000000000000",
      "soroban_mode": "horizon",
      "recorded_at": "2026-05-17T17:20:00Z",
      "explorer_url": "https://stellar.expert/explorer/testnet/tx/9b9fb616..."
    }
  ]
}
```

| Campo | Significado |
|-------|-------------|
| `kind` | `brlx_escrow_transfer`, `soroban_lock`, `anchor`, `off_ramp` (DB) ou `horizon_buyer`, `horizon_escrow_contract`, … (Horizon) |
| `explorer_url` | Link direto no [Stellar Expert testnet](https://stellar.expert/explorer/testnet) |
| `order_id` | UUID do pedido APICash (`00000000-…` quando a origem é só Horizon) |

Hashes `mock_*` e `mock_stellar_*` são **filtrados** e não aparecem na lista.

### Scripts (`money/apicash/scripts/`)

| Script | Função |
|--------|--------|
| `bootstrap-testnet-env.sh` | Gera/fundir identidades `holdfy-deployer`, `holdfy-buyer`, `holdfy-seller`; deploy token BRLx SAC; atualiza `money/.env` |
| `soroban-testnet-check.sh` | Valida `stellar` CLI, Wasm e variáveis obrigatórias |
| `soroban-testnet-deploy.sh` | Deploy do contrato escrow; imprime `APICASH_SOROBAN_ESCROW_CONTRACT_ID` |
| `validate-testnet-env.sh` | Usado pelo `runapp.sh` quando `APICASH_REQUIRE_TESTNET=1` |
| `testnet-onchain-smoke.sh` | Transfer BRLx → escrow + `lock` de teste na testnet |

### Arrancar só o core (API testnet)

Se `./runapp.sh start apicash` falhar noutros binários (ex. admin), pode subir só o core:

```bash
cd money/apicash
set -a && source ../.env && set +a
./target/debug/apicash-core
```

Depois: `curl -s http://127.0.0.1:3000/health` e `curl -s http://127.0.0.1:3000/testnet/transactions?limit=10`.

## Git e ficheiros sensíveis

O `.gitignore` na **raiz** aplica-se a todo o monorepo. **Nunca commitar:**

- `money/.env` e quaisquer `.env` com segredos reais
- `**/github.txt` ou notas com passwords
- `**/.runapp/` (PIDs, logs)
- `**/target/`, builds Flutter (`build/`, `.dart_tool/`)
- `*.tar.gz`, certificados (`*.pem`, `*.key`)

Templates versionados: `money/.env.example`, `**/.env.example`.

## Estrutura Git

```bash
git status          # raiz: pos-nearx/
git log -1
```

Histórico anterior só em `money/apicash` foi fundido neste repo; não há sub-repositórios aninhados.

## Git remoto

Repositório GitHub (SSH com host `github-holdfy`):

```bash
git remote -v
# origin  git@github-holdfy:holdfy/holdy.git

git push -u origin main
```

Chave SSH dedicada: `~/.ssh/holdfy` (config em `~/.ssh/config`, host `github-holdfy`).
