# pos-nearx

Monorepo da stack **HoldFy / NearX**: pagamentos PIX (Gatebox), plataforma fintech **APICash** (Rust + Stellar/Soroban), app banco (Flutter), wallet Go e utilitários de pareamento WhatsApp.

Um único repositório Git na raiz; configuração partilhada em `money/.env`.

## Mapa do repositório

```
pos-nearx/
├── money/                    # Orquestração: Docker, .env, runapp / runinfra
│   ├── apicash/              # Workspace Rust (core, admin, frontend Leptos, WhatsApp, Soroban)
│   ├── docker-compose.yml    # Postgres único, Redis, Pulsar
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
| Postgres (apicash, banco_saczuck, dubai-cash) | 5432 |
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

## x402 (HTTP 402 — Base Sepolia)

O projeto suporta o protocolo [x402](https://www.x402.org) para **micropagamentos por request HTTP** (USDC na **Base Sepolia**), em paralelo com o escrow **Stellar/BRLx** usado pelo WhatsApp e PIX.

| Via | Uso |
|-----|-----|
| **JWT** | Bot WhatsApp, app com login (`Authorization: Bearer …`) — **sem** cobrança x402 |
| **`/internal/*`** | Agente com `X-API-Key` — **isento** de x402 |
| **x402** | Integradores sem conta: recebem `402`, pagam USDC, repetem com `PAYMENT-SIGNATURE` |

Activar em `money/.env` (neste ambiente **x402 ainda não está activo** — só Stellar abaixo tem evidências reais):

```bash
APICASH_X402_REQUIRED=1
X402_FACILITATOR_URL=https://facilitator.x402.rs
X402_NETWORK=base-sepolia
X402_PAY_TO=<endereço_EVM_treasury_Base_Sepolia>
X402_PRICE_USDC=0.01
APICASH_PUBLIC_BASE_URL=http://127.0.0.1:3000
```

Validação e smoke:

```bash
cd money/apicash
scripts/validate-x402-env.sh
./runapp.sh start apicash   # ou cargo run -p apicash-core
scripts/x402-smoke.sh       # espera HTTP 402 em GET /orders/{id} sem auth
```

APIs x402 (quando activo):

| Serviço | URL / comportamento |
|---------|---------------------|
| **Facilitator** | `GET/POST` em `X402_FACILITATOR_URL` (ex. `https://facilitator.x402.rs`) — verificação e settlement USDC |
| **apicash-core** | Rotas protegidas sem JWT → `402 Payment Required` + header `Payment-Required` (v2) |
| **Rede** | Base Sepolia (`X402_NETWORK=base-sepolia`), treasury `X402_PAY_TO` |

## Stellar testnet

### Documentação em PDF (papel da Stellar no Holdfy)

Relatório com resumo do monorepo, **diagrama de sequência** (PIX → BRLx → escrow → release) e **diagrama de arquitetura** (APICash vs Gatebox). Saída: `docs/stellar-papel-no-projeto-holdfy.pdf`.

Requer Python 3 e `reportlab` (`pip install reportlab`). Gerar ou atualizar:

```bash
python3 /home/holdfy/git/holdy/docs/generate_stellar_pdf.py
```

(Equivalente, na raiz do repositório: `python3 docs/generate_stellar_pdf.py`.)

A **Stellar testnet** é a rede pública da Stellar Development Foundation — **não** é `127.0.0.1`. Os endpoints oficiais:

| Serviço | URL pública |
|---------|-------------|
| Horizon (REST) | `https://horizon-testnet.stellar.org` |
| Soroban RPC | `https://soroban-testnet.stellar.org` |
| Explorador | `https://stellar.expert/explorer/testnet` |

O `apicash-core` em `http://127.0.0.1:3000` (`API_PORT=3000` no `.env` actual) é só a **API local** deste projeto; ela pode **consultar** a testnet via Horizon, mas a ledger vive nos URLs acima.

Configuração em `money/.env` (valores **reais** usados nos smokes e curls abaixo; segredos `S…` ficam só no `.env`, não no Git):

| Variável | Valor real |
|----------|------------|
| `APICASH_STELLAR_NETWORK` | `testnet` |
| `APICASH_REQUIRE_TESTNET` | `1` |
| `APICASH_SOROBAN_ENABLED` | `1` |
| `APICASH_SOROBAN_STRICT` | `1` |
| `APICASH_STELLAR_HORIZON_URL` | `https://horizon-testnet.stellar.org` |
| `APICASH_SOROBAN_RPC_URL` | `https://soroban-testnet.stellar.org` |
| `APICASH_STELLAR_NETWORK_PASSPHRASE` | `Test SDF Network ; September 2015` |
| `APICASH_STELLAR_ASSET_CODE` | `BRLx` |
| `APICASH_STELLAR_CLI_BIN` | `stellar` |
| Identidades CLI (ficheiro local Stellar) | `holdfy-deployer`, `holdfy-buyer`, `holdfy-seller` |
| `APICASH_BRLX_TOKEN_CONTRACT_ID` | `CB5LOROS47TTN4SW3ZTXKEX2OLZHAGESNR4AX7XMOMX77LINIGUKW5MU` |
| `APICASH_SOROBAN_ESCROW_CONTRACT_ID` | `CDR6IJEPGRT5REHWS7ATKZVS4SEIGRNPK7LYRVF32KUERDWRDZQS6E7B` |
| `APICASH_STELLAR_BUYER_ADDRESS` | `GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC` |
| `APICASH_STELLAR_SELLER_ADDRESS` | `GAD43W2TSM47WXQTPER5FZLXQ6SZI7U74VW3WPFYYBFVFUYJOQ6IXSXA` |
| `APICASH_SOROBAN_ADMIN_ADDRESS` | `GAWOE7WXLVJA3CT6U4QJHGLJBAH47MPVU3TB7XIG2BGV2EIHYFPHFIYT` |
| `APICASH_SOROBAN_PLATFORM_ADDRESS` | `GAWOE7WXLVJA3CT6U4QJHGLJBAH47MPVU3TB7XIG2BGV2EIHYFPHFIYT` |
| `API_PORT` (apicash-core local) | `3000` |
| `APICASH_FIAT_RAIL` | `simulated` |
| `GATEBOX_BASE_URL` | `http://192.168.0.10:8081` |
| `MONEY_LAN_HOST` | `192.168.0.10` |

**Smoke on-chain** (`scripts/testnet-onchain-smoke.sh`):

| Parâmetro | Valor real |
|-----------|------------|
| Comando | `scripts/testnet-onchain-smoke.sh 5000000 99` |
| `amount` (arg 1) | `5000000` (= **0,5 BRLx**, 7 casas decimais) |
| `order_id` (arg 2) | `99` |
| `transfer` | `from` `GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC` → `to` `CDR6IJEPGRT5REHWS7ATKZVS4SEIGRNPK7LYRVF32KUERDWRDZQS6E7B` |
| Token invocado | `CB5LOROS47TTN4SW3ZTXKEX2OLZHAGESNR4AX7XMOMX77LINIGUKW5MU` |
| Assinatura CLI | `--source holdfy-buyer` |
| `lock` (contrato) | `CDR6IJEPGRT5REHWS7ATKZVS4SEIGRNPK7LYRVF32KUERDWRDZQS6E7B` — `buyer` `GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC`, `seller` `GAD43W2TSM47WXQTPER5FZLXQ6SZI7U74VW3WPFYYBFVFUYJOQ6IXSXA`, `token` `CB5LOROS47TTN4SW3ZTXKEX2OLZHAGESNR4AX7XMOMX77LINIGUKW5MU`, `amount` `5000000` |

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

Segredos de assinatura (`APICASH_SOROBAN_SOURCE_SECRET`, `APICASH_SOROBAN_BUYER_SOURCE`, etc.) — apenas em `money/.env`, **nunca** no README.

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

### Evidências on-chain (smoke `testnet-onchain-smoke.sh`)

Transações reais na testnet (`successful: true` no Horizon). Comando que as gerou:

```bash
cd money/apicash
scripts/testnet-onchain-smoke.sh 5000000 99
```

| # | Tipo | Hash | Ledger | UTC |
|---|------|------|--------|-----|
| 1 | Transfer BRLx (0,5 BRLx → escrow) | `7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649` | 2606118 | 2026-05-17T17:42:45Z |
| 2 | `lock` Soroban (`order_id=99`) | `b814f23a245abbd42dcff5343886c841aa2bd58858681ebae798a686316952a8` | 2606120 | 2026-05-17T17:42:55Z |

Explorador (testnet pública):  
https://stellar.expert/explorer/testnet/tx/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649

---

### Consultar a testnet — Horizon (fonte da verdade)

Qualquer máquina com `curl` acede à ledger **sem** subir o APICash.

#### 1) Detalhe de uma transação (transfer do smoke)

**Request:**

```bash
curl -sS 'https://horizon-testnet.stellar.org/transactions/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649'
```

**Response** (campos principais; resposta real da testnet):

```json
{
  "hash": "7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649",
  "successful": true,
  "ledger": 2606118,
  "created_at": "2026-05-17T17:42:45Z",
  "source_account": "GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC",
  "operation_count": 1,
  "_links": {
    "self": {
      "href": "https://horizon-testnet.stellar.org/transactions/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649"
    },
    "operations": {
      "href": "https://horizon-testnet.stellar.org/transactions/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649/operations{?cursor,limit,order}"
    }
  }
}
```

#### 2) `lock` Soroban (segunda tx do smoke)

**Request:**

```bash
curl -sS 'https://horizon-testnet.stellar.org/transactions/b814f23a245abbd42dcff5343886c841aa2bd58858681ebae798a686316952a8'
```

**Response** (campos principais):

```json
{
  "hash": "b814f23a245abbd42dcff5343886c841aa2bd58858681ebae798a686316952a8",
  "successful": true,
  "ledger": 2606120,
  "created_at": "2026-05-17T17:42:55Z",
  "source_account": "GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC",
  "operation_count": 1
}
```

#### 3) Últimas transações da conta comprador na testnet

**Request:**

```bash
curl -sS 'https://horizon-testnet.stellar.org/accounts/GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC/transactions?order=desc&limit=2'
```

**Response** (trecho `_embedded.records`; resposta real):

```json
{
  "_embedded": {
    "records": [
      {
        "hash": "b814f23a245abbd42dcff5343886c841aa2bd58858681ebae798a686316952a8",
        "successful": true,
        "ledger": 2606120,
        "created_at": "2026-05-17T17:42:55Z",
        "source_account": "GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC"
      },
      {
        "hash": "7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649",
        "successful": true,
        "ledger": 2606118,
        "created_at": "2026-05-17T17:42:45Z",
        "source_account": "GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC"
      }
    ]
  }
}
```

#### 4) Operações Soroban de uma tx (opcional)

**Request:**

```bash
curl -sS 'https://horizon-testnet.stellar.org/transactions/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649/operations'
```

**Response** (primeira operação, tipo típico):

```json
{
  "_embedded": {
    "records": [
      {
        "type": "invoke_host_function",
        "transaction_successful": true,
        "source_account": "GCAQCUWHUNZT5LLHVFN3V2PFAXLCH2CC6KSCTJRILZHXQRTFNWNJPIMC"
      }
    ]
  }
}
```

---

### API local APICash — espelho Horizon (opcional)

Só funciona com **`apicash-core` a correr** no host (`API_PORT`, por defeito **3000**). Esta URL **não é** a testnet; é um atalho que lê `https://horizon-testnet.stellar.org` e devolve JSON simplificado.

```http
GET http://127.0.0.1:3000/testnet/transactions?limit=10&source=horizon
```

| Query | Defeito | Descrição |
|-------|---------|-----------|
| `limit` | `10` | 1–50 |
| `source` | `all` | `horizon` — contas do `.env`; `db` — hashes nos pedidos; `all` — união |

**Request:**

```bash
curl -sS 'http://127.0.0.1:3000/testnet/transactions?limit=2&source=horizon'
```

(`API_PORT=3000` — ajuste o host/porta se o teu `.env` for diferente.)

**Response** (capturada com `apicash-core` a correr e `horizon_url` = `https://horizon-testnet.stellar.org`):

```json
{
  "network": "testnet",
  "horizon_url": "https://horizon-testnet.stellar.org",
  "limit": 2,
  "count": 2,
  "transactions": [
    {
      "hash": "b814f23a245abbd42dcff5343886c841aa2bd58858681ebae798a686316952a8",
      "kind": "horizon_buyer",
      "order_id": "00000000-0000-0000-0000-000000000000",
      "soroban_mode": "horizon",
      "recorded_at": "2026-05-17T17:42:55Z",
      "explorer_url": "https://stellar.expert/explorer/testnet/tx/b814f23a245abbd42dcff5343886c841aa2bd58858681ebae798a686316952a8"
    },
    {
      "hash": "7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649",
      "kind": "horizon_buyer",
      "order_id": "00000000-0000-0000-0000-000000000000",
      "soroban_mode": "horizon",
      "recorded_at": "2026-05-17T17:42:45Z",
      "explorer_url": "https://stellar.expert/explorer/testnet/tx/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649"
    }
  ]
}
```

| Campo | Significado |
|-------|-------------|
| `kind` | Origem: `horizon_buyer`, `brlx_escrow_transfer`, `soroban_lock`, … |
| `explorer_url` | Link Stellar Expert testnet |
| `order_id` | UUID do pedido APICash; `00000000-…` quando só veio do Horizon |

Hashes `mock_*` / `mock_stellar_*` são filtrados.

> **x402** (Base Sepolia) é outra rede — ver secção [x402](#x402-http-402--base-sepolia). Os exemplos acima são só **Stellar testnet** via Horizon.

### Scripts (`money/apicash/scripts/`)

| Script | Função |
|--------|--------|
| `bootstrap-testnet-env.sh` | Gera/fundir identidades `holdfy-deployer`, `holdfy-buyer`, `holdfy-seller`; deploy token BRLx SAC; atualiza `money/.env` |
| `soroban-testnet-check.sh` | Valida `stellar` CLI, Wasm e variáveis obrigatórias |
| `soroban-testnet-deploy.sh` | Deploy do contrato escrow; imprime `APICASH_SOROBAN_ESCROW_CONTRACT_ID` |
| `validate-testnet-env.sh` | Usado pelo `runapp.sh` quando `APICASH_REQUIRE_TESTNET=1` |
| `validate-x402-env.sh` | Valida `X402_*` quando `APICASH_X402_REQUIRED=1` |
| `testnet-onchain-smoke.sh` | Transfer BRLx → escrow + `lock` de teste na testnet |
| `x402-smoke.sh` | Espera HTTP 402 em rota protegida sem JWT/pagamento |

### Arrancar só o core (API testnet)

Se `./runapp.sh start apicash` falhar noutros binários (ex. admin), pode subir só o core:

```bash
cd money/apicash
set -a && source ../.env && set +a
./target/debug/apicash-core
```

Depois, API local: `curl -sS http://127.0.0.1:3000/health`. Para a testnet pública use Horizon (exemplos acima), por exemplo  
`curl -sS 'https://horizon-testnet.stellar.org/transactions/7dbe5d3a202a1eead07374e4bcc205ca73f3d8a8c5013105f4d0e3c3e82b8649'`.

## Ficheiros sensíveis

O `.gitignore` na **raiz** aplica-se a todo o monorepo. **Não versionar:**

- `money/.env` e quaisquer `.env` com segredos reais
- `**/github.txt` ou notas com passwords
- `**/.runapp/` (PIDs, logs)
- `**/target/`, builds Flutter (`build/`, `.dart_tool/`)
- `*.tar.gz`, certificados (`*.pem`, `*.key`)

Templates de referência: `money/.env.example`, `**/.env.example`.




cargo install --git https://github.com/rtk-ai/rtk
rtk init -g          # configura hook pro Claude Code
rtk gain             # confirma que instalou o certo