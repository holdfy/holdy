#!/bin/bash
#
# Sobe a infra Gatebox usando o único compose do workspace em money/docker-compose.yml
# (Postgres único, Redis e Pulsar partilhados com APICash).
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

SERVICES=(postgres redis pulsar)

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

POSTGRES_CTN="apicash-postgres"
POSTGRES_USER="${POSTGRES_USER:-apicash}"
GATEBOX_DB="${GATEBOX_POSTGRES_DB:-dubai-cash}"
REDIS_CTN="apicash-redis"
PULSAR_CTN="apicash-pulsar"

wait_until "Postgres" 90 docker exec "$POSTGRES_CTN" pg_isready -U "$POSTGRES_USER"
wait_until "Redis" 60 docker exec "$REDIS_CTN" redis-cli ping
wait_until "Pulsar" 240 docker exec "$PULSAR_CTN" bin/pulsar-admin brokers healthcheck

echo ""
echo "🗄️  Garantindo database Gatebox (${GATEBOX_DB})..."
DB_EXISTS=$(
  docker exec "$POSTGRES_CTN" psql -U "$POSTGRES_USER" -d postgres -tAc \
    "SELECT 1 FROM pg_database WHERE datname='${GATEBOX_DB}'" 2>/dev/null | tr -d '[:space:]' || true
)
if [[ "$DB_EXISTS" != "1" ]]; then
  docker exec "$POSTGRES_CTN" psql -U "$POSTGRES_USER" -d postgres -v ON_ERROR_STOP=1 \
    -c "CREATE DATABASE \"${GATEBOX_DB}\" OWNER ${POSTGRES_USER};"
  echo "✅ Database ${GATEBOX_DB} criada."
else
  echo "✅ Database ${GATEBOX_DB} já existe."
fi

echo ""
echo "🗄️  Verificando schema do Postgres..."
HAS_TX=$(
  docker exec "$POSTGRES_CTN" psql -U "$POSTGRES_USER" -d "$GATEBOX_DB" -tAc \
    "SELECT 1 FROM information_schema.tables WHERE table_schema='public' AND table_name='transaction' LIMIT 1;" \
    2>/dev/null || true
)
if [[ "$HAS_TX" == "1" ]]; then
  echo "✅ Schema já inicializado (transaction existe)."
else
  echo "⚠️  Schema base não encontrado. Aplicando db/*.sql..."
  for f in create-table.sql insert.sql seed-key-pix.sql; do
    [[ -f "${ROOT_DIR}/db/${f}" ]] || continue
    docker exec -i "$POSTGRES_CTN" psql -U "$POSTGRES_USER" -d "$GATEBOX_DB" -v ON_ERROR_STOP=1 \
      < "${ROOT_DIR}/db/${f}"
  done
  echo "✅ Schema base aplicado."
fi

echo ""
echo "✅ Infra pronta."
echo ""
echo "URLs/portas úteis (host):"
echo "  - Postgres:  localhost:5432 (db ${GATEBOX_DB}, user ${POSTGRES_USER})"
echo "  - Redis:     localhost:6379"
echo "  - Pulsar:    pulsar://localhost:6650 (admin: http://localhost:8080)"
if [ "$WITH_OBSERVABILITY" = "1" ]; then
  echo "  - Prometheus: http://localhost:9091"
  echo "  - Grafana:    http://localhost:3001 (admin/admin)"
fi
