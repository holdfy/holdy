# APICash — comandos de desenvolvimento (GNU Make)
SHELL := /bin/bash
.DEFAULT_GOAL := help

.PHONY: help setup-env build build-all test test-all fmt clippy clean \
	up down logs \
	db-migrate \
	run-core run-admin run-whatsapp run-frontend dev run-all \
	deploy-contracts soroban-check soroban-deploy \
	runinfra runapp app-status infra-status

help:
	@echo "APICash — comandos úteis"
	@echo "  make up / down / logs   - delega para ../runinfra.sh na pasta money/"
	@echo "  make db-migrate         - sqlx migrate run (requer sqlx-cli + Postgres up)"
	@echo "  make run-core           - API principal (porta $$(echo $${API_PORT:-3000}))"
	@echo "  make run-admin          - backend admin (porta $$(echo $${ADMIN_PORT:-3001}))"
	@echo "  make run-whatsapp       - agente WhatsApp"
	@echo "  make run-frontend       - UI Leptos SSR"
	@echo "  make dev / run-all      - core + admin + whatsapp em paralelo (Node npx ou script)"
	@echo "  make build-all / test-all — alias para build e test do workspace"
	@echo "  make deploy-contracts   - cargo run -p apicash-cli -- deploy-contracts (Soroban)"
	@echo "  make soroban-check/deploy - valida/deploy Soroban testnet"
	@echo "  make runinfra / runapp  - infra local e ../runapp.sh (apps na pasta money/)"
	@echo "  make build / test / fmt / clippy / clean"
	@echo "  make setup-env         - ../setup-env.sh (money/.env + symlinks + runinfra)"

setup-env:
	@cd .. && ./setup-env.sh

build:
	cargo build --workspace

build-all: build

test:
	cargo test --workspace

test-all: test

deploy-contracts:
	cargo run -p apicash-cli -- deploy-contracts

soroban-check:
	./scripts/soroban-testnet-check.sh

soroban-deploy:
	./scripts/soroban-testnet-deploy.sh

runinfra:
	@cd .. && ./runinfra.sh

runapp:
	@../runapp.sh

infra-status:
	@cd .. && ./runinfra.sh status

app-status:
	@../runapp.sh status

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

clean:
	cargo clean

# --- Infra Docker: só existe em ../ (money/docker-compose.yml + ../runinfra.sh) ---
up:
	@cd .. && ./runinfra.sh start

down:
	@cd .. && ./runinfra.sh stop

logs:
	@cd .. && ./runinfra.sh logs

# Aliases retrocompatíveis
docker-up: up
docker-down: down
docker-logs: logs

# --- SQLx (opcional) ---
db-migrate:
	@set -a; [ -f ../.env ] && . ../.env; set +a; \
	export DATABASE_URL="$${DATABASE_URL:-postgresql://$${POSTGRES_USER:-apicash}:$${POSTGRES_PASSWORD:-apicash}@127.0.0.1:$${POSTGRES_PORT:-5432}/$${POSTGRES_DB:-apicash}}"; \
	if [ ! -d migrations ] || [ -z "$$(find migrations -maxdepth 1 -name '*.sql' 2>/dev/null | head -1)" ]; then \
	  echo "[db-migrate] Nenhum ficheiro .sql em migrations/ — nada a aplicar."; \
	  exit 0; \
	fi; \
	echo "[db-migrate] DATABASE_URL=$$DATABASE_URL"; \
	if command -v sqlx >/dev/null 2>&1; then \
	  sqlx migrate run; \
	elif command -v docker >/dev/null 2>&1 && (cd .. && docker compose --project-directory "$$(pwd)" -f docker-compose.yml $$(test -f ../.env && echo --env-file ../.env) ps -q postgres | grep -q .); then \
	  echo "[db-migrate] sqlx-cli ausente; fallback via docker compose exec postgres psql"; \
	  for f in migrations/*.sql; do \
	    echo "[db-migrate] aplicando $$f"; \
	    ( cd .. && docker compose --project-directory "$$(pwd)" -f docker-compose.yml $$(test -f ../.env && echo --env-file ../.env) exec -T postgres psql -U "$${POSTGRES_USER:-apicash}" -d "$${POSTGRES_DB:-apicash}" -v ON_ERROR_STOP=1 ) < "$$f"; \
	  done; \
	else \
	  echo "[db-migrate] Instale: cargo install sqlx-cli --no-default-features --features rustls,postgres"; \
	  exit 1; \
	fi

# --- Serviços Rust ---
run-core:
	set -a; [ -f ../.env ] && . ../.env; set +a; \
	export APICASH_HTTP_PORT="$${APICASH_HTTP_PORT:-$${API_PORT:-3000}}"; \
	cargo run -p apicash-core

run-admin:
	set -a; [ -f ../.env ] && . ../.env; set +a; \
	cargo run -p apicash-admin-backend

run-whatsapp:
	set -a; [ -f ../.env ] && . ../.env; set +a; \
	cargo run -p apicash-whatsapp

run-frontend:
	cargo run -p apicash-frontend --features ssr

# Paralelo: tenta npx concurrently (sem package.json); senão scripts/dev_parallel.sh
dev: up
	@if command -v npx >/dev/null 2>&1; then \
	  echo "[dev] A usar npx concurrently@9 …"; \
	  npx --yes concurrently@9 \
	    -n core,admin,wa \
	    -c blue,green,yellow \
	    "bash -lc 'set -a; [ -f ../.env ] && . ../.env; set +a; cargo run -p apicash-core'" \
	    "bash -lc 'set -a; [ -f ../.env ] && . ../.env; set +a; cargo run -p apicash-admin-backend'" \
	    "bash -lc 'set -a; [ -f ../.env ] && . ../.env; set +a; cargo run -p apicash-whatsapp'"; \
	else \
	  echo "[dev] npx não encontrado — a usar scripts/dev_parallel.sh"; \
	  bash scripts/dev_parallel.sh; \
	fi

# Core + admin + WhatsApp (sem Docker; mesmo padrão que dev)
run-all:
	@if command -v npx >/dev/null 2>&1; then \
	  echo "[run-all] npx concurrently@9 …"; \
	  npx --yes concurrently@9 \
	    -n core,admin,wa \
	    -c blue,green,yellow \
	    "bash -lc 'set -a; [ -f ../.env ] && . ../.env; set +a; cargo run -p apicash-core'" \
	    "bash -lc 'set -a; [ -f ../.env ] && . ../.env; set +a; cargo run -p apicash-admin-backend'" \
	    "bash -lc 'set -a; [ -f ../.env ] && . ../.env; set +a; cargo run -p apicash-whatsapp'"; \
	else \
	  echo "[run-all] npx não encontrado — a usar make dev (infra) ou três terminais"; \
	  exit 1; \
	fi
