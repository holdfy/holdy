#!/bin/bash
#
# Sobe a infra Gatebox usando o único compose do workspace em money/docker-compose.yml
# (Postgres Gatebox, Redis partilhado APICash, Pulsar Gatebox).
#
# Uso:
#   ./scripts/start-infra.sh
#   COMPOSE_FILE=/caminho/docker-compose.yml ./scripts/start-infra.sh   # override raro
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MONEY_DIR="$(cd "$ROOT_DIR/../.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-$MONEY_DIR/docker-compose.yml}"

WITH_OBSERVABILITY="${WITH_OBSERVABILITY:-0}"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║        🧱 Iniciando Infra (para Gatebox Rust)                 ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

echo "🐳 Verificando Docker..."
if ! docker info &>/dev/null; then
  echo "❌ Docker não está rodando. Inicie o Docker primeiro."
  exit 1
fi
echo "✅ Docker OK"
echo ""

if [[ ! -f "$COMPOSE_FILE" ]]; then
  echo "❌ Ficheiro compose não encontrado: $COMPOSE_FILE"
  echo "   Este projeto usa apenas: ${MONEY_DIR}/docker-compose.yml"
  exit 1
fi

SERVICES=(gatebox-postgres redis gatebox-pulsar)

if [ "$WITH_OBSERVABILITY" = "1" ]; then
  echo "⚠️  Prometheus/Grafana não fazem parte do compose em money/ — ignorando WITH_OBSERVABILITY."
fi

echo "📦 Compose: $COMPOSE_FILE"
echo "📦 Subindo containers: ${SERVICES[*]}"
docker compose --project-directory "$MONEY_DIR" -f "$COMPOSE_FILE" up -d "${SERVICES[@]}"

echo ""
echo "⏳ Aguardando serviços ficarem saudáveis..."

wait_until() {
  local name="$1"
  local timeout_s="$2"
  shift 2
  local start
  start="$(date +%s)"
  while true; do
    if "$@" &>/dev/null; then
      echo "✅ $name - OK"
      return 0
    fi
    local now
    now="$(date +%s)"
    if [ $((now - start)) -ge "$timeout_s" ]; then
      echo "❌ $name não ficou pronto em ${timeout_s}s"
      return 1
    fi
    sleep 2
  done
}

POSTGRES_CTN="gateboxrust_postgres"
REDIS_CTN="apicash-redis"
PULSAR_CTN="gateboxrust_pulsar"

wait_until "Postgres" 90 docker exec "$POSTGRES_CTN" pg_isready -U postgres
wait_until "Redis" 60 docker exec "$REDIS_CTN" redis-cli ping
wait_until "Pulsar" 240 docker exec "$PULSAR_CTN" bin/pulsar-admin brokers healthcheck

# Inicializa schema base quando o DB estiver vazio (necessário para a API Rust sair do health-only).
# Obs: o arquivo está dentro do projeto gatebox-rust e é montado no container em /db/create-table.sql.
echo ""
echo "🗄️  Verificando schema do Postgres..."
HAS_TX=$(
  docker exec "$POSTGRES_CTN" psql -U postgres -d dubai-cash -tAc \
    "SELECT 1 FROM information_schema.tables WHERE table_schema='public' AND table_name='transaction' LIMIT 1;" \
    2>/dev/null || true
)
if [[ "$HAS_TX" == "1" ]]; then
  echo "✅ Schema já inicializado (transaction existe)."
else
  echo "⚠️  Schema base não encontrado. Aplicando /db/create-table.sql..."
  docker exec "$POSTGRES_CTN" psql -U postgres -d dubai-cash -f /db/create-table.sql
  echo "✅ Schema base aplicado."
fi

echo ""
echo "🌱 Verificando seed DEV..."
SEED_COUNT=$(
  docker exec "$POSTGRES_CTN" psql -U postgres -d dubai-cash -tAc \
    "SELECT count(1) FROM account_status_types;" \
    2>/dev/null || true
)
if [[ "${SEED_COUNT:-0}" == "0" ]]; then
  echo "⚠️  Seed não encontrado. Aplicando /db/insert.sql..."
  docker exec "$POSTGRES_CTN" psql -U postgres -d dubai-cash -f /db/insert.sql
  echo "✅ Seed aplicado."
else
  echo "✅ Seed já aplicado (account_status_types count=${SEED_COUNT})."
fi

echo ""
echo "🔑 Garantindo key_pix do simulador..."
docker exec "$POSTGRES_CTN" psql -U postgres -d dubai-cash -f /db/seed-key-pix.sql >/dev/null
echo "✅ key_pix OK."

echo ""
echo "✅ Infra pronta."
echo ""
echo "URLs/portas úteis (host):"
echo "  - Postgres:  localhost:5433 (db dubai-cash, user postgres, pass root)"
echo "  - Redis:     localhost:6379"
echo "  - Pulsar:    pulsar://localhost:6651 (admin: http://localhost:8082)"
if [ "$WITH_OBSERVABILITY" = "1" ]; then
  echo "  - Prometheus: http://localhost:9091"
  echo "  - Grafana:    http://localhost:3001 (admin/admin)"
fi
