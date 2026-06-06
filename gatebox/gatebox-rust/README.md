# Gatebox Rust

API em Rust (equivalente funcional ao stack original em Go). Este crate vive no monorepo **`money/`**, que tem **um único** `docker-compose.yml` e **um** `runapp.sh` na raiz de `money/`.

## Pré-requisitos

- Rust (stable)
- Para feature Pulsar: `protoc` (ex.: `apt-get install protobuf-compiler`)

## Configuração

```bash
# Preferir o .env unificado do workspace (em money/)
# Ver: money/setup-env.sh
./scripts/setup-env.sh   # apenas se mantiver .env local ao crate; no fluxo típico usa-se money/.env
```

## Build e testes

```bash
cargo build
cargo test
```

## Rodar localmente

**Recomendado (stack completa):** a partir da pasta `money/`:

```bash
./runinfra.sh          # ou: docker compose up -d
./runapp.sh start gatebox
```

Isto sobe a infra (Postgres único :5432, Redis e **o mesmo Pulsar** que o APICash em `money/docker-compose.yml`) e arranca Gatebox-Rust via `money/runapp.sh`.

Para **só** a API Rust, com a infra já em execução:

```bash
./scripts/run-rust.sh
```

## Docker (imagem só da API)

```bash
# Build (sem Pulsar)
docker build -t gatebox-rust .

# Build com feature Pulsar
docker build --build-arg PULSAR=1 -t gatebox-rust .

# Rodar (passe .env ou variáveis)
docker run --rm -p 8080:8080 -p 2112:2112 --env-file .env gatebox-rust
```

## Variáveis principais

| Variável | Descrição |
|----------|-----------|
| `PORT` | Porta HTTP (default 8080) |
| `POSTGRESQL_WRITE_URL` / `POSTGRESQL_READ_URL` | Conexão Postgres |
| `JWT_SECRET` | Chave para JWT |
| `REDIS_URL` | Redis (opcional) |
| `MESSAGING_BACKEND` | `pulsar` (preferido neste projeto) ou `rabbitmq` se usar AMQP externo |
| `PULSAR_URL` | Broker Pulsar (no stack `money/`: mesmo que APICash, ex. `pulsar://127.0.0.1:6650`) |
| `ENABLE_METRICS` | true/false — servidor Prometheus na porta 2112 |
| `METRICS_PORT` | Porta das métricas (default 2112) |
| `PIX_GATEWAY_SULCRED` | Se definido, usa Sulcred para PIX |
| `SULCRED_CLIENT_ID` / `SULCRED_CLIENT_SECRET` | Credenciais Sulcred |

Ver `.env.example` e `money/.env.example` para a lista completa alinhada ao workspace.

## Endpoints Admin (front-gatebox)

- `GET /admin/reports/profit` – Lucro da plataforma (TTO+TPO)
- `POST /admin/customers/:id/account` – Criar conta para cliente
- `GET /admin/customers/:id/balance` – Saldo do cliente
- `GET /admin/customers/:id/extract` – Extrato do cliente
