# Backend Banco Saczuck

Backend bancario sandbox independente do Gatebox, com dominio, autenticacao e API proprios.

## Stack
- Go 1.24+
- `net/http` com `ServeMux` (Go 1.22+ patterns)
- Postgres do `docker-compose.yml` raiz (sem novo container de banco)

## Estrutura
- `cmd/server`: bootstrap HTTP
- `internal/config`: configuracoes/env
- `internal/database`: conexao + migrations
- `internal/models`: estados e entidades
- `internal/repositories`: acesso ao banco
- `internal/services`: regras de negocio
- `internal/controllers`: camada HTTP
- `internal/gatebox_client`: integracao isolada com API do Gatebox
- `docs`: documentacao API

## Rodando
```bash
cp .env.example .env
go mod tidy
go run ./cmd/server
```

## Endpoints principais
- `POST /accounts`
- `POST /auth/login`
- `GET /accounts/me`
- `GET /accounts/me/balance`
- `POST /accounts/me/topup`
- `GET /transactions`
- `POST /payments/validate`
- `POST /payments/pix`
- `POST /payments/qrcode`
- `POST /payments/link`
- `POST /payments/{id}/approve`
- `POST /payments/{id}/reject`
- `POST /payments/{id}/pending`
- `POST /payments/{id}/refund`
- `POST /webhooks/gatebox`
- `GET /simulation/settings`
- `PUT /simulation/settings`

