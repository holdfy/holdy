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
- **Go** 1.22+ (wallet, `backend_banco`)
- **Flutter** (opcional, `gatebox/banco/app_banco`)
- **Node/npm** (opcional, front Gatebox e `make dev` do APICash)
- `protoc` se compilar Gatebox com feature Pulsar

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

## Remote (quando estiver pronto)

```bash
git remote add origin <url>
git push -u origin main
```
