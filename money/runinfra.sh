#!/usr/bin/env bash
#
# Infra Docker unificada (money/docker-compose.yml) + migrações SQLx (APICash + Gatebox Rust opcional).
#
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${MONEY}"

APICASH="${MONEY}/apicash"
COMPOSE_FILE="${MONEY}/docker-compose.yml"

compose_money() {
  local ef=()
  [[ -f "${MONEY}/.env" ]] && ef+=(--env-file "${MONEY}/.env")
  docker compose --project-directory "${MONEY}" -f "${COMPOSE_FILE}" "${ef[@]}" "$@"
}

log() {
  printf '[money/runinfra] %s\n' "$*"
}

warn() {
  printf '[money/runinfra][warn] %s\n' "$*" >&2
}

load_env() {
  set -a
  [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
  set +a

  export MONEY_LAN_HOST="${MONEY_LAN_HOST:-192.168.86.64}"
  export POSTGRES_USER="${POSTGRES_USER:-apicash}"
  export POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-apicash}"
  export POSTGRES_DB="${POSTGRES_DB:-apicash}"
  export POSTGRES_PORT="${POSTGRES_PORT:-5432}"
  export DATABASE_URL="${DATABASE_URL:-postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${MONEY_LAN_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}}"
}

need_docker() {
  command -v docker >/dev/null 2>&1 || {
    warn "docker not found"
    exit 1
  }
  docker compose version >/dev/null 2>&1 || {
    warn "docker compose not available"
    exit 1
  }
}

stop_apps_if_running() {
  if [ -x "${MONEY}/runapp.sh" ]; then
    log "stopping host apps (APICash / Gatebox) before infra restart"
    "${MONEY}/runapp.sh" stop apicash >/dev/null 2>&1 || true
    "${MONEY}/runapp.sh" stop gatebox >/dev/null 2>&1 || true
  fi
}

stop_infra() {
  load_env
  need_docker
  stop_apps_if_running
  log "stopping docker compose (${COMPOSE_FILE})"
  compose_money down --remove-orphans
}

start_infra() {
  load_env
  need_docker
  log "starting docker compose (${COMPOSE_FILE})"
  compose_money up -d
  wait_infra
  migrate_gatebox_db
  migrate_db
  wait_gatebox_apps_hint
}

wait_one_container() {
  local service="$1"
  local timeout="${2:-180}"
  local id status health

  id="$(compose_money ps -q "${service}" 2>/dev/null || true)"
  if [ -z "${id}" ]; then
    warn "${service}: no container id"
    return 1
  fi

  local _
  for _ in $(seq 1 "${timeout}"); do
    status="$(docker inspect -f '{{.State.Status}}' "${id}" 2>/dev/null || true)"
    health="$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "${id}" 2>/dev/null || true)"

    case "${health}" in
    healthy)
      log "${service}: healthy"
      return 0
      ;;
    none)
      if [ "${status}" = "running" ]; then
        log "${service}: running"
        return 0
      fi
      ;;
    unhealthy)
      warn "${service}: unhealthy"
      compose_money logs --tail=80 "${service}" || true
      return 1
      ;;
    esac

    sleep 1
  done

  warn "${service}: timeout waiting for healthy/running"
  compose_money logs --tail=80 "${service}" || true
  return 1
}

wait_infra() {
  log "waiting for infrastructure health"
  wait_one_container postgres 120
  wait_one_container redis 120
  wait_one_container nats 30
}

gatebox_pg_url() {
  load_env
  local u pw port db
  u="${POSTGRES_USER:-apicash}"
  pw="${POSTGRES_PASSWORD:-apicash}"
  port="${POSTGRES_PORT:-5432}"
  db="${GATEBOX_POSTGRES_DB:-dubai-cash}"
  printf 'postgres://%s:%s@%s:%s/%s?sslmode=disable\n' "${u}" "${pw}" "${MONEY_LAN_HOST:-192.168.86.64}" "${port}" "${db}"
}

gatebox_rust_root_with_cargo() {
  local _p _candidates=()
  [[ -n "${GATEBOX_RUST_DIR:-}" ]] && _candidates+=("${GATEBOX_RUST_DIR}")
  _candidates+=("/home/devel/git/pos-nearx/gatebox/gatebox-rust")
  _candidates+=("${MONEY}/../gatebox/gatebox-rust")
  _candidates+=("${MONEY}/gatebox/gatebox-rust")

  for _p in "${_candidates[@]}"; do
    [[ -z "${_p}" ]] && continue
    [[ -f "${_p}/Cargo.toml" ]] || continue
    printf '%s\n' "$(cd "${_p}" && pwd)"
    return 0
  done
  return 1
}

# Schema base Gatebox (db/*.sql) quando a base dubai-cash ainda não tem tabela transaction.
# As migrações SQLx do crate assumem o schema base (tabela transaction, etc.).
bootstrap_gatebox_sql_if_needed() {
  load_env
  need_docker
  local root pguser gbdb has_tx
  root="$(gatebox_rust_root_with_cargo 2>/dev/null)" || return 0
  [[ -n "${root:-}" ]] || return 0
  pguser="${POSTGRES_USER:-apicash}"
  gbdb="${GATEBOX_POSTGRES_DB:-dubai-cash}"

  has_tx="$(
    compose_money exec -T postgres psql -U "${pguser}" -d "${gbdb}" -tAc \
      "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public' AND table_name='transaction'" 2>/dev/null | tr -d '[:space:]' || echo 0
  )"

  if [[ "${has_tx}" == "1" ]]; then
    return 0
  fi

  local f
  for f in "${root}/db/create-table.sql" "${root}/db/insert.sql" "${root}/db/seed-key-pix.sql"; do
    [[ -f "${f}" ]] || continue
    log "Gatebox database ${gbdb}: applying $(basename "${f}") (base schema / seed)"
    compose_money exec -T postgres psql -U "${pguser}" -d "${gbdb}" -v ON_ERROR_STOP=1 <"${f}" || {
      warn "bootstrap gatebox failed on $(basename "${f}")"
      return 1
    }
  done
}

ensure_gatebox_db() {
  load_env
  need_docker
  local dbname="${GATEBOX_POSTGRES_DB:-dubai-cash}"
  log "ensuring postgres database for gatebox: ${dbname}"
  local exists
  exists="$(
    compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d postgres -tAc \
      "SELECT 1 FROM pg_database WHERE datname='${dbname}'" 2>/dev/null | tr -d '[:space:]' || true
  )"
  if [ "${exists}" = "1" ]; then
    log "database ${dbname} already exists"
    return 0
  fi
  compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d postgres -v ON_ERROR_STOP=1 \
    -c "CREATE DATABASE \"${dbname}\" OWNER ${POSTGRES_USER};"
  log "created database ${dbname}"
}

migrate_gatebox_db() {
  load_env
  local root
  root="$(gatebox_rust_root_with_cargo 2>/dev/null)" || {
    log "Gatebox Rust não encontrado — skip migrações"
    return 0
  }
  if [[ ! -d "${root}/migrations" ]] || [[ -z "$(compgen -G "${root}/migrations/*.sql" || true)" ]]; then
    log "Gatebox: sem migrations/*.sql — skip"
    return 0
  fi

  need_docker
  ensure_gatebox_db
  bootstrap_gatebox_sql_if_needed || return 1

  local durl="${POSTGRESQL_WRITE_URL:-}"
  if [[ -z "${durl}" ]]; then
    durl="$(gatebox_pg_url)"
  fi

  log "running database migrations (gatebox-rust)"
  if command -v sqlx >/dev/null 2>&1; then
    (cd "${root}" && DATABASE_URL="${durl}" sqlx migrate run) || {
      warn "sqlx migrate (gatebox) failed"
      return 1
    }
    return 0
  fi

  warn "sqlx CLI ausente — a API Gatebox aplica migrações no arranque; ou instale sqlx-cli"
}

wait_gatebox_apps_hint() {
  load_env
  if root="$(gatebox_rust_root_with_cargo 2>/dev/null)" && [[ -n "${root:-}" ]]; then
    log "Gatebox database OK (Postgres único) — NATS JetStream (serviço nats); API Rust: ./runapp.sh start gatebox (ou start all)"
  fi
}

# Base dedicada ao backend_banco (migrations em Go ao arranque; ver money/runapp.sh).
ensure_banco_db() {
  load_env
  need_docker
  local dbname="${BANCO_POSTGRES_DB:-banco_saczuck}"
  log "ensuring postgres database for backend_banco: ${dbname}"
  local exists
  exists="$(
    compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d postgres -tAc \
      "SELECT 1 FROM pg_database WHERE datname='${dbname}'" 2>/dev/null | tr -d '[:space:]' || true
  )"
  if [ "${exists}" = "1" ]; then
    log "database ${dbname} already exists"
    return 0
  fi
  compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d postgres -v ON_ERROR_STOP=1 \
    -c "CREATE DATABASE ${dbname};"
  log "created database ${dbname}"
}

migrate_db() {
  load_env
  ensure_banco_db
  if [ ! -d "${APICASH}/migrations" ] || [ -z "$(compgen -G "${APICASH}/migrations/*.sql" || true)" ]; then
    log "no migrations directory/files; skipping db migration"
    return 0
  fi

  log "running database migrations (apicash)"
  (
    cd "${APICASH}"
    if command -v sqlx >/dev/null 2>&1; then
      DATABASE_URL="${DATABASE_URL}" sqlx migrate run
      return 0
    fi

    warn "sqlx not found; applying migrations via docker compose exec psql"
    local f
    for f in migrations/*.sql; do
      log "applying ${f}"
      compose_money exec -T postgres psql \
        -U "${POSTGRES_USER}" \
        -d "${POSTGRES_DB}" \
        -v ON_ERROR_STOP=1 <"${f}"
    done
  )
}

check_postgres() {
  compose_money exec -T postgres pg_isready -U "${POSTGRES_USER:-apicash}" -d "${POSTGRES_DB:-apicash}" >/dev/null 2>&1
}

check_redis() {
  compose_money exec -T redis redis-cli ping 2>/dev/null | grep -q PONG
}

check_nats() {
  load_env
  local port="${NATS_MONITOR_PORT:-8222}"
  command -v curl >/dev/null 2>&1 || return 1
  curl -sf --max-time 2 "http://127.0.0.1:${port}/healthz" >/dev/null
}

print_status() {
  load_env
  need_docker

  printf '\n== Docker compose (money/) ==\n'
  compose_money ps

  printf '\n== Health checks ==\n'
  if check_postgres; then
    printf '  postgres: ok\n'
  else
    printf '  postgres: fail\n'
  fi

  if check_redis; then
    printf '  redis   : ok\n'
  else
    printf '  redis   : fail\n'
  fi

  if check_nats; then
    printf '  nats    : ok\n'
  else
    printf '  nats    : fail\n'
  fi

  printf '\n== Database (Postgres único :%s) ==\n' "${POSTGRES_PORT:-5432}"
  printf '  DATABASE_URL=%s\n' "${DATABASE_URL}"
  if compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -tAc "select count(*) from information_schema.tables where table_schema = 'public';" >/tmp/apicash_table_count 2>/dev/null; then
    printf '  %s public tables=%s\n' "${POSTGRES_DB:-apicash}" "$(tr -d '[:space:]' </tmp/apicash_table_count)"
  else
    printf '  %s public tables=unknown\n' "${POSTGRES_DB:-apicash}"
  fi
  local gbdb="${GATEBOX_POSTGRES_DB:-dubai-cash}"
  if compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d "${gbdb}" -tAc "select count(*) from information_schema.tables where table_schema = 'public';" >/tmp/gatebox_table_count 2>/dev/null; then
    printf '  %s public tables=%s\n' "${gbdb}" "$(tr -d '[:space:]' </tmp/gatebox_table_count)"
  else
    printf '  %s public tables=unknown\n' "${gbdb}"
  fi
  local bancodb="${BANCO_POSTGRES_DB:-banco_saczuck}"
  if compose_money exec -T postgres psql -U "${POSTGRES_USER}" -d "${bancodb}" -tAc "select count(*) from information_schema.tables where table_schema = 'public';" >/tmp/banco_table_count 2>/dev/null; then
    printf '  %s public tables=%s\n' "${bancodb}" "$(tr -d '[:space:]' </tmp/banco_table_count)"
  else
    printf '  %s public tables=unknown\n' "${bancodb}"
  fi
}

logs_infra() {
  need_docker
  compose_money logs -f
}

usage() {
  cat <<USAGE
Usage: ${MONEY}/runinfra.sh [restart|start|stop|status|migrate|logs]

Default: restart

Compose: ${COMPOSE_FILE}
Env: ${MONEY}/.env

Infra cria a base Postgres ${BANCO_POSTGRES_DB:-banco_saczuck} (backend_banco em gatebox/banco/backend_banco).

Recomendação: ${MONEY}/setup-env.sh cria symlink + sobe infra (runinfra).

Ou pedaços:
  ${MONEY}/runinfra.sh [start|stop|status]
  ${MONEY}/runapp.sh start
USAGE
}

cmd="${1:-restart}"
case "${cmd}" in
restart)
  stop_infra
  start_infra
  print_status
  ;;
start)
  start_infra
  print_status
  ;;
stop)
  stop_infra
  print_status
  ;;
status)
  print_status
  ;;
migrate)
  load_env
  need_docker
  migrate_gatebox_db
  migrate_db
  print_status
  ;;
logs)
  logs_infra
  ;;
-h | --help | help)
  usage
  ;;
*)
  usage
  exit 2
  ;;
esac
