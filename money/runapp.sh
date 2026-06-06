#!/usr/bin/env bash
#
# Arranque das apps do workspace: APICash (core :3000, admin :3001, frontend :3002, whatsapp :3010),
# site público Vite (holdy/site :5173), Gatebox Rust (PIX), opcionalmente backend_banco Go.
# Gatebox: gatebox/gatebox-rust (symlink criado por setup-env.sh para ../gatebox se existir) ou GATEBOX_RUST_DIR.
# Infra Docker: money/docker-compose.yml (Postgres único + Redis/Pulsar partilhados).
#
# Integração WhatsApp (B) → apicash-core → âncora → Gatebox PIX EMV:
#   · APICash: GATEBOX_BASE_URL, APICASH_GATEBOX_ENABLED; gateways PIX (Sulcred/etc.) só no Gatebox — ver gatebox-rust + money/.env partilhado
#   · `./runapp.sh start all`: sobe gatebox-rust antes do APICash se RUNAPP_AUTO_GATEBOX=1 (não arranca simuladores de gateway)
#   · Sem Gatebox ativo + GATEBOX_BASE_URL, o rail simulado não obtém PIX (pedidos falham ao criar instrução).
#
# Uso:
#   ./runapp.sh                    # = stop all + start all (scope all, default)
#   ./runapp.sh restart [scope]    # idem: para tudo no scope e sobe de novo (+ build no start)
#   ./runapp.sh stop [apicash|gatebox|banco|site|all]   (stop apicash não para o Gatebox)
#   ./runapp.sh start [apicash|gatebox|banco|site|all]
#   ./runapp.sh status
#   ./runapp.sh build [apicash|banco|site|all]
#   ./runapp.sh logs [apicash|gatebox|banco|site|all]
#
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Node/bun para holdy/site (Vite) — disponível em todo o script.
export PATH="${HOME}/.bun/bin:${HOME}/.local/bin:/usr/local/bin:${PATH}"
# Host usado em URLs (Postgres, Pulsar, APIs, Gatebox, Sulcred). Sobrescreva com MONEY_LAN_HOST em money/.env
export MONEY_LAN_HOST="${MONEY_LAN_HOST:-192.168.0.10}"
# Health checks no mesmo host: usar loopback evita falhar quando MONEY_LAN_HOST não é o IP desta máquina.
export RUNAPP_LOOPBACK="${RUNAPP_LOOPBACK:-127.0.0.1}"
APICASH="${MONEY}/apicash"

# backend_banco Go: normalmente money/gatebox/banco/backend_banco (symlink setup-env) ou repo irmão ../gatebox/banco/backend_banco
resolve_banco_backend_dir() {
  if [ -n "${BANCO_BACKEND_DIR:-}" ] && [ -d "${BANCO_BACKEND_DIR}" ]; then
    printf '%s\n' "${BANCO_BACKEND_DIR}"
    return
  fi
  local nested="${MONEY}/gatebox/banco/backend_banco"
  local sibling="${MONEY}/../gatebox/banco/backend_banco"
  if [ -d "${nested}" ]; then
    printf '%s\n' "${nested}"
    return
  fi
  if [ -d "${sibling}" ]; then
    (cd "${sibling}" && pwd)
    return
  fi
  printf '%s\n' "${nested}"
}
BANCO_BE="$(resolve_banco_backend_dir)"
BB_LOG="${MONEY}/.runapp/banco"
BB_BIN="${BB_LOG}/banco-backend"

COMPOSE_FILE="${MONEY}/docker-compose.yml"

AC_RUN="${APICASH}/.runapp"
AC_PID="${AC_RUN}/pids"
AC_LOG="${AC_RUN}/logs"
mkdir -p "${AC_PID}" "${AC_LOG}"

GB_LOG="${MONEY}/.runapp/gatebox"
GB_PID="${GB_LOG}/gatebox-rust.pid"
mkdir -p "${GB_LOG}"

# Se 1 (defeito com gatebox presente no disco), `./runapp.sh start all` sobe também a API Rust Gatebox antes do APICash.
RUNAPP_AUTO_GATEBOX="${RUNAPP_AUTO_GATEBOX:-1}"
# Se 1 (defeito), `./runapp.sh start all` sobe também o site Vite em holdy/site (npm run dev).
RUNAPP_AUTO_SITE="${RUNAPP_AUTO_SITE:-1}"
# Modo de rede: testnet | mainnet | "" (vazio = usa o .env sem alteração)
RUNAPP_NETWORK_MODE="${RUNAPP_NETWORK_MODE:-}"

resolve_site_dir() {
  if [ -n "${SITE_DIR:-}" ] && [ -d "${SITE_DIR}" ]; then
    printf '%s\n' "${SITE_DIR}"
    return
  fi
  local sibling="${MONEY}/../site"
  if [ -d "${sibling}" ]; then
    (cd "${sibling}" && pwd)
    return
  fi
  printf '%s\n' "${sibling}"
}
SITE_ROOT="$(resolve_site_dir)"
SITE_PORT="${SITE_PORT:-5173}"
SITE_LOG="${MONEY}/.runapp/site"
mkdir -p "${SITE_LOG}"

HOLDFY_ADMIN_ROOT="${HOLDFY_ADMIN_ROOT:-${MONEY}/../holdfy-admin}"
HOLDFY_ADMIN_PORT="${HOLDFY_ADMIN_PORT:-3020}"
HOLDFY_ADMIN_LOG="${MONEY}/.runapp/holdfy-admin"
mkdir -p "${HOLDFY_ADMIN_LOG}"

FRONT_GATEBOX_ROOT="${FRONT_GATEBOX_ROOT:-${MONEY}/../gatebox/front-gatebox}"
FRONT_GATEBOX_PORT="${FRONT_GATEBOX_PORT:-3030}"
FRONT_GATEBOX_LOG="${MONEY}/.runapp/front-gatebox"
mkdir -p "${FRONT_GATEBOX_LOG}"

SCRAPER_ROOT="${SCRAPER_ROOT:-${MONEY}/../scraper-service}"
SCRAPER_PORT="${SCRAPER_PORT:-4000}"
SCRAPER_LOG="${MONEY}/.runapp/scraper-service"
mkdir -p "${SCRAPER_LOG}"

RASTREIO_ROOT="${RASTREIO_ROOT:-${MONEY}/../apprastreio/backend}"
RASTREIO_PORT="${LOGISTICA_HTTP_PORT:-${RASTREIO_PORT:-8092}}"
RASTREIO_LOG="${MONEY}/.runapp/rastreio"
RASTREIO_BIN="${RASTREIO_LOG}/logistica-holdfy-backend"
mkdir -p "${RASTREIO_LOG}"

# WhatsApp pareamento: pasta do PNG + pair.html para o browser. Sobrescrever com WA_QR_DIR no .env
WA_QR_DIR_DEFAULT="${WA_QR_DIR_DEFAULT:-$(dirname "${MONEY}")/whatsapp_qrcode}"

# Gatebox ports (prefixo GB_* para não colidir com API_PORT do .env do APICash)
GB_API_PORT="${GB_API_PORT:-8081}"
GB_METRICS_PORT="${GB_METRICS_PORT:-2112}"
CLIENT_SIM_PORT="${CLIENT_SIM_PORT:-7070}"
SEVENTRUST_PORT="${SEVENTRUST_PORT:-7010}"
GATEWAY_URL="${GATEWAY_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"
WEBHOOK_URL="${WEBHOOK_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"

# Compose money: Postgres único (5432); Pulsar e Redis partilhados. scripts/start-infra.sh só se UNIFIED_SKIP_GATEBOX_INFRA=0.
UNIFIED_SKIP_GATEBOX_INFRA="${UNIFIED_SKIP_GATEBOX_INFRA:-1}"

log() { printf '[money/runapp] %s\n' "$*" >&2; }
warn() { printf '[money/runapp][warn] %s\n' "$*" >&2; }

# Abre URL no browser do ambiente gráfico (xdg-open / gio).
runapp_open_in_browser() {
  local label="$1"
  local url="$2"
  if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    warn "sem DISPLAY/WAYLAND — abra manualmente (${label}): ${url}"
    return 0
  fi
  if command -v xdg-open >/dev/null 2>&1; then
    log "abrindo ${label} no browser: ${url}"
    xdg-open "${url}" >/dev/null 2>&1 &
  elif command -v gio >/dev/null 2>&1; then
    log "abrindo ${label} (gio): ${url}"
    gio open "${url}" >/dev/null 2>&1 &
  else
    warn "xdg-open/gio ausente — abra no browser (${label}): ${url}"
  fi
}

# Abre duas URLs em abas separadas (xdg-open em sequência reutiliza a mesma aba).
runapp_open_two_browser_tabs() {
  local qr_url="${1:-}"
  local site_url="${2:-}"
  if [ -z "${qr_url}" ] && [ -z "${site_url}" ]; then
    return 0
  fi
  if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    [ -n "${qr_url}" ] && warn "sem DISPLAY/WAYLAND — QR: ${qr_url}"
    [ -n "${site_url}" ] && warn "sem DISPLAY/WAYLAND — site: ${site_url}"
    return 0
  fi

  local browser="" b
  for b in google-chrome google-chrome-stable chromium chromium-browser brave-browser microsoft-edge vivaldi; do
    if command -v "${b}" >/dev/null 2>&1; then
      browser="${b}"
      break
    fi
  done

  if [ -n "${browser}" ]; then
    if [ -n "${qr_url}" ] && [ -n "${site_url}" ]; then
      log "abrindo QR: ${qr_url}"
      log "abrindo site: ${site_url}"
      "${browser}" --new-tab "${qr_url}" >/dev/null 2>&1 &
      sleep 1
      "${browser}" --new-tab "${site_url}" >/dev/null 2>&1 &
    elif [ -n "${qr_url}" ]; then
      log "abrindo QR: ${qr_url}"
      "${browser}" --new-tab "${qr_url}" >/dev/null 2>&1 &
    elif [ -n "${site_url}" ]; then
      log "abrindo site: ${site_url}"
      "${browser}" --new-tab "${site_url}" >/dev/null 2>&1 &
    fi
    return 0
  fi

  if command -v firefox >/dev/null 2>&1; then
    if [ -n "${qr_url}" ] && [ -n "${site_url}" ]; then
      log "abrindo QR: ${qr_url}"
      log "abrindo site: ${site_url}"
      firefox -new-tab "${qr_url}" >/dev/null 2>&1 &
      sleep 1
      firefox -new-tab "${site_url}" >/dev/null 2>&1 &
    elif [ -n "${qr_url}" ]; then
      firefox -new-tab "${qr_url}" >/dev/null 2>&1 &
    elif [ -n "${site_url}" ]; then
      firefox -new-tab "${site_url}" >/dev/null 2>&1 &
    fi
    return 0
  fi

  if [ -n "${qr_url}" ]; then
    log "abrindo pareamento WhatsApp: ${qr_url}"
    if command -v xdg-open >/dev/null 2>&1; then
      xdg-open "${qr_url}" >/dev/null 2>&1 &
    elif command -v gio >/dev/null 2>&1; then
      gio open "${qr_url}" >/dev/null 2>&1 &
    fi
    sleep 2
  fi
  if [ -n "${site_url}" ]; then
    log "abrindo site HoldFy: ${site_url}"
    if command -v xdg-open >/dev/null 2>&1; then
      xdg-open "${site_url}" >/dev/null 2>&1 &
    elif command -v gio >/dev/null 2>&1; then
      gio open "${site_url}" >/dev/null 2>&1 &
    fi
  fi
}

compose_money() {
  local ef=()
  [[ -f "${MONEY}/.env" ]] && ef+=(--env-file "${MONEY}/.env")
  docker compose --project-directory "${MONEY}" -f "${COMPOSE_FILE}" "${ef[@]}" "$@"
}

# -----------------------------------------------------------------------------
# APICash
# -----------------------------------------------------------------------------

truthy() {
  case "${1:-}" in
  1 | true | TRUE | yes | YES | on | ON) return 0 ;;
  *) return 1 ;;
  esac
}

port_from_bind() {
  local bind="$1"
  printf '%s\n' "${bind##*:}"
}

ac_pidfile() { printf '%s/%s.pid\n' "${AC_PID}" "$1"; }

ac_is_running_pid() {
  local pid="${1:-}"
  [ -n "${pid}" ] && kill -0 "${pid}" >/dev/null 2>&1
}

ac_kill_pid() {
  local name="$1"
  local pid="$2"
  ac_is_running_pid "${pid}" || return 0
  log "stopping ${name} pid=${pid}"
  kill -TERM "${pid}" >/dev/null 2>&1 || true
  local _
  for _ in $(seq 1 40); do
    ac_is_running_pid "${pid}" || return 0
    sleep 0.25
  done
  warn "force killing ${name} pid=${pid}"
  kill -KILL "${pid}" >/dev/null 2>&1 || true
}

ac_stop_pidfile() {
  local name="$1"
  local pf pid
  pf="$(ac_pidfile "${name}")"
  [ -f "${pf}" ] || return 0
  pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
  [ -n "${pid}" ] && ac_kill_pid "${name}" "${pid}"
  rm -f "${pf}"
}

ac_stop_by_pattern() {
  local name="$1"
  local pattern="$2"
  command -v pgrep >/dev/null 2>&1 || return 0
  local pids
  pids="$(pgrep -f "${pattern}" || true)"
  [ -n "${pids}" ] || return 0
  local pid
  while IFS= read -r pid; do
    [ -n "${pid}" ] || continue
    [ "${pid}" = "$$" ] && continue
    ac_kill_pid "${name}" "${pid}"
  done <<<"${pids}"
}

ac_stop_port() {
  local port="$1"
  [ -n "${port}" ] || return 0
  if command -v fuser >/dev/null 2>&1; then
    local pids
    pids="$(fuser "${port}/tcp" 2>/dev/null || true)"
    [ -n "${pids}" ] || return 0
    local pid
    while IFS= read -r pid; do
      [ -n "${pid}" ] || continue
      ac_kill_pid "port:${port}" "${pid}"
    done < <(printf '%s\n' ${pids})
  fi
}

ac_core_port() {
  port_from_bind "${APICASH_HTTP_BIND:-0.0.0.0:${APICASH_HTTP_PORT:-3000}}"
}

ac_wa_port() {
  port_from_bind "${APICASH_WA_WEBHOOK_BIND:-0.0.0.0:3010}"
}

ac_load_env() {
  set -a
  [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
  set +a

  export MONEY_LAN_HOST="${MONEY_LAN_HOST:-192.168.0.10}"

  export APICASH_HTTP_PORT="${APICASH_HTTP_PORT:-${API_PORT:-3000}}"
  export APICASH_HTTP_BIND="${APICASH_HTTP_BIND:-0.0.0.0:${APICASH_HTTP_PORT}}"
  export ADMIN_PORT="${ADMIN_PORT:-${APICASH_ADMIN_PORT:-3001}}"
  export APICASH_FRONTEND_PORT="${APICASH_FRONTEND_PORT:-3002}"
  export APICASH_WA_WEBHOOK_BIND="${APICASH_WA_WEBHOOK_BIND:-0.0.0.0:3010}"
  export APICASH_WA_TRANSPORT="${APICASH_WA_TRANSPORT:-rust}"
  export APICASH_WA_SQLITE_PATH="${APICASH_WA_SQLITE_PATH:-file:${APICASH}/.runapp/apicash_whatsapp.db}"
  WA_QR_DIR="${WA_QR_DIR:-${WA_QR_DIR_DEFAULT}}"
  export APICASH_WA_QR_PNG="${APICASH_WA_QR_PNG:-${WA_QR_DIR}/whatsapp-pairing-qr.png}"
  mkdir -p "${WA_QR_DIR}" "$(dirname "${APICASH_WA_QR_PNG}")" 2>/dev/null || true
  export APICASH_CORE_URL="${APICASH_CORE_URL:-http://${MONEY_LAN_HOST}:${APICASH_HTTP_PORT}}"
  # Rail PIX sandbox sem HTTP até `APICASH_STELLAR_ANCHOR_URL` ser um servidor real (+ `APICASH_FIAT_RAIL=anchor`).
  export APICASH_FIAT_RAIL="${APICASH_FIAT_RAIL:-simulated}"
  export GB_API_PORT="${GB_API_PORT:-8081}"
  export GATEBOX_BASE_URL="${GATEBOX_BASE_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"
  # defeito 1: comprador B recebe PIX via Gatebox (EMV). Defina 0 só para dev sem API Rust.
  export APICASH_GATEBOX_ENABLED="${APICASH_GATEBOX_ENABLED:-1}"
  export RUST_MIN_STACK="${RUST_MIN_STACK:-16777216}"
  case "${RUST_LOG:-}" in
  *apicash_whatsapp* | *whatsapp_rust*) ;;
  "")
    export RUST_LOG="info,apicash_core=info,apicash_admin_backend=info,tower_http=info,apicash_whatsapp=info,whatsapp_rust=info"
    ;;
  *)
    export RUST_LOG="${RUST_LOG},apicash_whatsapp=info,whatsapp_rust=info"
    ;;
  esac
}

ac_services() {
  printf '%s\n' \
    "apicash-core|target/debug/apicash-core|http://${RUNAPP_LOOPBACK:-127.0.0.1}:$(ac_core_port)/health|$(ac_core_port)" \
    "apicash-admin-backend|target/debug/apicash-admin-backend|http://${RUNAPP_LOOPBACK:-127.0.0.1}:${ADMIN_PORT:-3001}/health|${ADMIN_PORT:-3001}" \
    "apicash-frontend|target/debug/apicash-frontend|http://${RUNAPP_LOOPBACK:-127.0.0.1}:${APICASH_FRONTEND_PORT:-3002}/|${APICASH_FRONTEND_PORT:-3002}" \
    "apicash-whatsapp|target/debug/apicash-whatsapp|http://${RUNAPP_LOOPBACK:-127.0.0.1}:$(ac_wa_port)/health|$(ac_wa_port)"
}

ac_stop_all() {
  ac_load_env
  log "stopping APICash application processes"

  local name bin _health port
  while IFS='|' read -r name bin _health port; do
    ac_stop_pidfile "${name}"
    ac_stop_by_pattern "${name}" "${APICASH}/${bin}"
    ac_stop_by_pattern "${name}" "target/debug/${name}"
    ac_stop_by_pattern "${name}" "cargo run -p ${name}"
    ac_stop_port "${port}"
  done < <(ac_services)

  rm -f "${AC_PID}"/*.pid 2>/dev/null || true
}

resolve_stellar_identity_secret() {
  local value="${1:-}"
  [ -n "${value}" ] || return 1
  if [[ "${value}" == S* ]]; then
    printf '%s\n' "${value}"
    return 0
  fi
  command -v stellar >/dev/null 2>&1 || return 1
  stellar keys secret "${value}" 2>/dev/null || return 1
}

ac_validate_testnet_policy() {
  if ! truthy "${APICASH_REQUIRE_TESTNET:-}"; then
    return 0
  fi
  export APICASH_SOROBAN_ENABLED=1
  export APICASH_SOROBAN_STRICT=1
  export APICASH_STELLAR_NETWORK="${APICASH_STELLAR_NETWORK:-testnet}"
  if [ -x "${APICASH}/scripts/validate-testnet-env.sh" ]; then
    "${APICASH}/scripts/validate-testnet-env.sh"
  else
    warn "validate-testnet-env.sh não encontrado em ${APICASH}/scripts/"
    return 1
  fi
}

# Aplica overrides de rede por cima do .env. Chamado após ac_load_env para ter precedência.
apply_network_mode() {
  local mode="${1:-}"
  [ -n "${mode}" ] || return 0
  case "${mode}" in
  testnet)
    export APICASH_REQUIRE_TESTNET=1
    export APICASH_SOROBAN_ENABLED=1
    export APICASH_SOROBAN_STRICT=1
    export APICASH_STELLAR_NETWORK=testnet
    # Usar simulator_anchor_pix no lugar do Etherfuse real
    export APICASH_FIAT_RAIL=anchor
    export APICASH_STELLAR_ANCHOR_URL="http://${MONEY_LAN_HOST:-192.168.0.10}:${ANCHOR_SIM_PORT:-8093}"
    log "rede: testnet (SOROBAN_ENABLED=1, REQUIRE_TESTNET=1, STELLAR_NETWORK=testnet, ANCHOR_SIM=${APICASH_STELLAR_ANCHOR_URL})"
    ;;
  mainnet)
    export APICASH_REQUIRE_TESTNET=
    export APICASH_SOROBAN_ENABLED=1
    export APICASH_SOROBAN_STRICT=1
    export APICASH_STELLAR_NETWORK=mainnet
    log "rede: mainnet (SOROBAN_ENABLED=1, STELLAR_NETWORK=mainnet)"
    ;;
  *)
    warn "RUNAPP_NETWORK_MODE desconhecido: ${mode} (use testnet ou mainnet)"
    ;;
  esac
}

ac_prepare_runtime_env() {
  ac_load_env
  apply_network_mode "${RUNAPP_NETWORK_MODE:-}"

  export APICASH_ORDERS_PG="${APICASH_ORDERS_PG:-1}"
  export APICASH_CUSTODY_PG="${APICASH_CUSTODY_PG:-1}"
  export APICASH_SCORES_PG="${APICASH_SCORES_PG:-1}"
  export APICASH_ADMIN_PG="${APICASH_ADMIN_PG:-1}"

  if truthy "${APICASH_REQUIRE_TESTNET:-}"; then
    ac_validate_testnet_policy || {
      warn "APICASH_REQUIRE_TESTNET=1: corrija money/.env (scripts/soroban-testnet-deploy.sh) antes de subir APICash"
      return 1
    }
  fi

  if truthy "${APICASH_SOROBAN_ENABLED:-}"; then
    local buyer_secret dispute_secret
    if buyer_secret="$(resolve_stellar_identity_secret "${APICASH_SOROBAN_BUYER_SOURCE:-}")"; then
      export APICASH_SOROBAN_BUYER_SOURCE="${buyer_secret}"
    fi
    if dispute_secret="$(resolve_stellar_identity_secret "${APICASH_SOROBAN_DISPUTE_SOURCE:-}")"; then
      export APICASH_SOROBAN_DISPUTE_SOURCE="${dispute_secret}"
    fi
  fi
}

ac_build_all() {
  ac_prepare_runtime_env
  log "building APICash binaries"
  cd "${APICASH}"

  local core_args=(-p apicash-core)
  if truthy "${APICASH_SOROBAN_ENABLED:-}" || truthy "${APICASH_REQUIRE_TESTNET:-}"; then
    core_args+=(--features soroban)
  fi

  cargo build "${core_args[@]}"

  local admin_args=(-p apicash-admin-backend)
  if truthy "${APICASH_SOROBAN_ENABLED:-}" || truthy "${APICASH_REQUIRE_TESTNET:-}"; then
    admin_args+=(--features soroban)
  fi
  cargo build "${admin_args[@]}"

  cargo build -p apicash-frontend --no-default-features --features ssr
  cargo build -p apicash-whatsapp
}

ac_wait_http() {
  local name="$1"
  local url="$2"
  local attempts="${3:-120}"

  [ -n "${url}" ] || return 0
  command -v curl >/dev/null 2>&1 || return 0

  local _
  for _ in $(seq 1 "${attempts}"); do
    if curl -sS --max-time 1 "${url}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
  done

  warn "${name} did not answer ${url}; check logs in ${AC_LOG}"
  return 1
}

ac_start_one() {
  local name="$1"
  local bin="$2"
  local health="$3"

  [ -x "${APICASH}/${bin}" ] || {
    warn "missing binary ${bin}; run: ${MONEY}/runapp.sh build apicash"
    return 1
  }

  local out="${AC_LOG}/${name}.log"
  local pf
  pf="$(ac_pidfile "${name}")"

  log "starting ${name}"
  (
    cd "${APICASH}"
    exec "${APICASH}/${bin}"
  ) >>"${out}" 2>&1 &
  local pid=$!
  printf '%s\n' "${pid}" >"${pf}"

  sleep 0.5
  if ! ac_is_running_pid "${pid}"; then
    warn "${name} exited during startup; last log lines:"
    tail -n 40 "${out}" 2>/dev/null || true
    return 1
  fi

  ac_wait_http "${name}" "${health}" 80 || true
}

ac_start_all() {
  ac_prepare_runtime_env
  log "starting APICash application services"
  cd "${APICASH}"

  local name bin health _port
  while IFS='|' read -r name bin health _port; do
    ac_start_one "${name}" "${bin}" "${health}"
  done < <(ac_services)

  ac_verify_gatebox_integration_hint || true
  if ! truthy "${RUNAPP_DEFER_BROWSER:-0}"; then
    ac_maybe_open_whatsapp_pairing_browser || true
  fi
}

# Recria whatsapp_qrcode/ (pair.html + alinha PNG para o browser). Chamado em todo runapp.sh.
ac_setup_whatsapp_qr_dir() {
  ac_load_env 2>/dev/null || true
  local dir="${WA_QR_DIR:-${WA_QR_DIR_DEFAULT}}"
  local src_html="${MONEY}/scripts/whatsapp-pair.html"
  local dst_html="${dir}/pair.html"
  local browser_png="${dir}/whatsapp-pairing-qr.png"
  mkdir -p "${dir}" 2>/dev/null || true
  if [ -f "${src_html}" ]; then
    cp -f "${src_html}" "${dst_html}"
  else
    warn "template ${src_html} ausente — copie scripts/whatsapp-pair.html para ${dst_html}"
    return 1
  fi
  export APICASH_WA_QR_PNG="${APICASH_WA_QR_PNG:-${browser_png}}"
  # O HTML usa whatsapp-pairing-qr.png na mesma pasta; manter cópia/symlink alinhado ao env.
  if [ "${APICASH_WA_QR_PNG}" != "${browser_png}" ]; then
    if [ -s "${APICASH_WA_QR_PNG}" ]; then
      cp -f "${APICASH_WA_QR_PNG}" "${browser_png}" 2>/dev/null || true
    fi
    ln -sf "${APICASH_WA_QR_PNG}" "${browser_png}" 2>/dev/null || true
  fi
  if [ ! -s "${browser_png}" ] && [ -s "${APICASH}/.runapp/whatsapp-pairing-qr.png" ]; then
    cp -f "${APICASH}/.runapp/whatsapp-pairing-qr.png" "${browser_png}" 2>/dev/null || true
  fi
  touch "${dir}/.gitkeep" 2>/dev/null || true
  log "WhatsApp QR dir: ${dir} (abra file://${dst_html})"
}

# Apaga sessão SQLite e gera QR novo (browser + PNG em WA_QR_DIR).
ac_whatsapp_pair_reset() {
  ac_load_env
  ac_setup_whatsapp_qr_dir || true
  log "a parar apicash-whatsapp e apagar sessão (novo pareamento)"
  ac_stop_pidfile "apicash-whatsapp"
  ac_stop_by_pattern "apicash-whatsapp" "${APICASH}/target/debug/apicash-whatsapp"
  ac_stop_by_pattern "apicash-whatsapp" "target/debug/apicash-whatsapp"
  ac_stop_port "$(ac_wa_port)"
  local db_uri="${APICASH_WA_SQLITE_PATH:-file:${APICASH}/.runapp/apicash_whatsapp.db}"
  local db_path="${db_uri#file:}"
  rm -f "${db_path}" "${db_path}-wal" "${db_path}-shm" 2>/dev/null || true
  rm -f "${WA_QR_DIR}/whatsapp-pairing-qr.png" "${APICASH_WA_QR_PNG}" 2>/dev/null || true
  log "sessão removida; a subir apicash-whatsapp (aguarde o QR no browser)"
  ac_start_one "apicash-whatsapp" "target/debug/apicash-whatsapp" \
    "http://${RUNAPP_LOOPBACK:-127.0.0.1}:$(ac_wa_port)/health"
  ac_maybe_open_whatsapp_pairing_browser || true
}

# Página local (file://) com QR que renova — abre no browser após subir apicash-whatsapp.
ac_install_whatsapp_pair_html() {
  ac_setup_whatsapp_qr_dir
}

ac_whatsapp_qr_file_url() {
  ac_load_env
  case "${APICASH_WA_TRANSPORT:-rust}" in
  cloud) return 1 ;;
  esac
  truthy "${APICASH_WA_OPEN_BROWSER:-1}" || return 1
  ac_setup_whatsapp_qr_dir || return 1
  local html
  html="$(cd "${WA_QR_DIR}" && pwd)/pair.html"
  printf 'file://%s\n' "${html}"
}

ac_maybe_open_whatsapp_pairing_browser() {
  local qr_url
  qr_url="$(ac_whatsapp_qr_file_url 2>/dev/null)" || return 0
  runapp_open_in_browser "pareamento WhatsApp" "${qr_url}"
  local browser_png="${WA_QR_DIR}/whatsapp-pairing-qr.png"
  if [ ! -s "${browser_png}" ] && [ ! -s "${APICASH_WA_QR_PNG}" ]; then
    warn "QR ainda não gerado (sessão já pareada?). Para novo QR: ./runapp.sh whatsapp-pair"
  fi
}

runapp_open_startup_browsers() {
  local qr_url="" site_url=""
  ac_load_env

  if truthy "${RUNAPP_AUTO_SITE:-1}" && site_present && truthy "${RUNAPP_OPEN_SITE_BROWSER:-1}"; then
    site_url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${SITE_PORT}/"
    if command -v curl >/dev/null 2>&1 && ! curl -fsS --max-time 1 "${site_url}" >/dev/null 2>&1; then
      warn "site ainda não responde — a subir antes de abrir o browser"
      RUNAPP_DEFER_BROWSER=1 site_start_all || warn "site não subiu — ver ${SITE_LOG}/vite.log"
    fi
    if command -v curl >/dev/null 2>&1; then
      local waited=0
      while ! curl -fsS --max-time 1 "${site_url}" >/dev/null 2>&1; do
        if [ "${waited}" -ge 90 ]; then
          warn "site não respondeu — abri só o QR WhatsApp"
          site_url=""
          break
        fi
        sleep 1
        waited=$((waited + 1))
      done
    fi
  fi

  qr_url="$(ac_whatsapp_qr_file_url 2>/dev/null)" || qr_url=""

  if [ -n "${qr_url}" ] || [ -n "${site_url}" ]; then
    log "abrindo QR WhatsApp + site no browser (2 abas)"
    runapp_open_two_browser_tabs "${qr_url}" "${site_url}"
  fi

  if holdfy_admin_present; then
    local admin_url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${HOLDFY_ADMIN_PORT}/"
    runapp_open_in_browser "holdfy-admin (dashboard)" "${admin_url}" || true
    sleep 0.4
    runapp_open_in_browser "holdfy-admin (pedidos)" "${admin_url}orders" || true
    sleep 0.4
    runapp_open_in_browser "holdfy-admin (Stellar/Soroban)" "${admin_url}stellar" || true
  fi
  if front_gatebox_present; then
    sleep 0.4
    local fg_base="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${FRONT_GATEBOX_PORT}"
    log "abrindo GateBox portal"
    runapp_open_in_browser "GateBox admin" "${fg_base}/#/admin/login" || true
    sleep 0.4
    runapp_open_in_browser "GateBox transações HoldFy" "${fg_base}/#/admin/holdfy/transactions" || true
  fi
  if rastreio_present; then
    sleep 0.4
    local rastreio_url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${RASTREIO_PORT}/"
    runapp_open_in_browser "Rastreio (logistica-holdfy)" "${rastreio_url}trackers" || true
  fi

  local dev_dash="${MONEY}/../dev-dashboard.html"
  if [ -f "${dev_dash}" ]; then
    sleep 0.4
    local dash_dir
    dash_dir="$(cd "$(dirname "${dev_dash}")" && pwd)"
    local dash_port="${DEV_DASH_PORT:-9090}"
    # Serve via HTTP so browser CORS allows the health-check fetch() calls.
    if ! fuser "${dash_port}/tcp" >/dev/null 2>&1; then
      mkdir -p "${MONEY}/.runapp/dev-dash"
      python3 -m http.server "${dash_port}" --directory "${dash_dir}" \
        >"${MONEY}/.runapp/dev-dash/server.log" 2>&1 &
      echo $! > "${MONEY}/.runapp/dev-dash/server.pid"
      sleep 0.3
    fi
    runapp_open_in_browser "referência dev Holdfy" "http://127.0.0.1:${dash_port}/dev-dashboard.html" || true
  fi

  if [ -n "${qr_url}" ]; then
    local browser_png="${WA_QR_DIR}/whatsapp-pairing-qr.png"
    if [ ! -s "${browser_png}" ] && [ ! -s "${APICASH_WA_QR_PNG:-}" ]; then
      warn "QR ainda não gerado (sessão já pareada?). Para novo QR: ./runapp.sh whatsapp-pair"
    fi
  fi
}

site_maybe_open_browser() {
  if ! truthy "${RUNAPP_OPEN_SITE_BROWSER:-1}"; then
    return 0
  fi
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${SITE_PORT}/"
  command -v curl >/dev/null 2>&1 || {
    warn "curl ausente — não abri o browser do site (${url})"
    return 0
  }
  local waited=0
  while ! curl -fsS --max-time 1 "${url}" >/dev/null 2>&1; do
    if [ "${waited}" -ge 90 ]; then
      warn "site não respondeu — não abri o browser (${url})"
      return 0
    fi
    sleep 1
    waited=$((waited + 1))
  done
  runapp_open_in_browser "site HoldFy" "${url}"
}

# Após subir o core: avisa se PIX pelo Gatebox está ligado mas a API não responde.
ac_verify_gatebox_integration_hint() {
  ac_load_env
  truthy "${APICASH_GATEBOX_ENABLED:-}" || return 0
  command -v curl >/dev/null 2>&1 || return 0
  local base="${GATEBOX_BASE_URL:-}"
  [[ -n "${base}" ]] || return 0
  base="${base%/}"
  if curl -fsS --max-time 2 "${base}/health" >/dev/null 2>&1; then
    log "Gatebox OK (${base}/health) — rail simulado usará EMV para pedidos / WhatsApp"
    return 0
  fi
  warn "APICASH_GATEBOX_ENABLED=1 mas Gatebox não responde em ${base}/health — pedidos falham ao gerar PIX. Suba: ./runapp.sh start gatebox"
}

ac_http_code() {
  local url="$1"
  local code
  [ -n "${url}" ] || {
    printf -- '-'
    return 0
  }
  command -v curl >/dev/null 2>&1 || {
    printf 'curl-missing'
    return 0
  }
  code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "${url}" 2>/dev/null || true)"
  printf '%s' "${code:-000}"
}

ac_print_status() {
  ac_load_env
  printf '\n== APICash (applications) ==\n'
  local name _bin health port pf pid state http log_path
  while IFS='|' read -r name _bin health port; do
    pf="$(ac_pidfile "${name}")"
    log_path="${AC_LOG}/${name}.log"
    pid="-"
    state="stopped"
    if [ -f "${pf}" ]; then
      pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
      if ac_is_running_pid "${pid}"; then
        state="running"
      else
        state="stale-pid"
      fi
    fi
    http="$(ac_http_code "${health}")"
    printf '  %-24s %-10s pid=%-8s port=%-5s http=%-4s log=%s\n' "${name}" "${state}" "${pid}" "${port}" "${http}" "${log_path}"
  done < <(ac_services)
}

ac_logs_all() {
  ac_load_env
  mkdir -p "${AC_LOG}"
  log "following APICash logs; Ctrl+C to stop"
  tail -n 80 -F "${AC_LOG}"/*.log
}

# -----------------------------------------------------------------------------
# Gatebox Rust (PIX — porta GB_API_PORT; PULSAR_URL igual ao APICash salvo GATEBOX_PULSAR_CLIENT_URL)
# -----------------------------------------------------------------------------

gb_kill_by_port() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    if lsof -ti:"${port}" >/dev/null 2>&1; then
      # shellcheck disable=SC2046
      lsof -ti:"${port}" | xargs kill -9 2>/dev/null || true
    fi
    return 0
  fi
  if command -v fuser >/dev/null 2>&1; then
    fuser -k "${port}/tcp" >/dev/null 2>&1 || true
    return 0
  fi
  warn "nem lsof nem fuser disponíveis; não foi possível libertar porta ${port}"
}

gb_wait_http_ok() {
  local name="$1"
  local url="$2"
  local timeout_s="${3:-60}"
  local start now
  start="$(date +%s)"
  while true; do
    if curl -fsS "${url}" >/dev/null 2>&1; then
      log "${name} OK (${url})"
      return 0
    fi
    now="$(date +%s)"
    if [ $((now - start)) -ge "${timeout_s}" ]; then
      warn "timeout esperando ${name}: ${url}"
      return 1
    fi
    sleep 2
  done
}

gb_gatebox_rust_dir() {
  # Exige Cargo.toml — evita apanhar pasta-stub `./gatebox/gatebox-rust` criada só com `db/` para o bind-mount Docker.
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

gb_load_env() {
  set -a
  [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
  set +a
  export MONEY_LAN_HOST="${MONEY_LAN_HOST:-192.168.0.10}"
  export GB_API_PORT="${GB_API_PORT:-8081}"
  export GB_METRICS_PORT="${GB_METRICS_PORT:-2112}"
  export GATEBOX_BASE_URL="${GATEBOX_BASE_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"
}

gb_maybe_start_infra_gatebox() {
  local root scripts
  root="$(gb_gatebox_rust_dir)" || return 0
  scripts="${root}/scripts/start-infra.sh"
  truthy "${UNIFIED_SKIP_GATEBOX_INFRA:-1}" && return 0
  [[ -x "${scripts}" ]] || [[ -f "${scripts}" ]] || return 0
  log "UNIFIED_SKIP_GATEBOX_INFRA=0 — executing ${scripts} (compose money: Postgres único + Pulsar)"
  (cd "${root}" && bash "${scripts}") || warn "start-infra Gatebox terminou com erro (ignorando se já há containers)"
}

gb_stop_all() {
  local gbroot
  gbroot="$(gb_gatebox_rust_dir 2>/dev/null)" || true
  log "stopping Gatebox Rust (gatebox-rust)"

  local pid
  if [[ -f "${GB_PID}" ]]; then
    pid="$(tr -d '[:space:]' <"${GB_PID}" 2>/dev/null || true)"
    [[ -n "${pid}" ]] && ac_kill_pid "gatebox-rust" "${pid}"
    rm -f "${GB_PID}"
  fi

  ac_stop_by_pattern "gatebox-rust" "target/release/gatebox-rust"
  ac_stop_by_pattern "gatebox-rust" "target/debug/gatebox-rust"
  ac_stop_by_pattern "gatebox-rust" "cargo run.*gatebox-rust"
  gb_kill_by_port "${GB_API_PORT}"

  [[ -n "${gbroot:-}" ]] && gb_kill_by_port "${GB_METRICS_PORT}"
}

gb_build() {
  local root
  root="$(gb_gatebox_rust_dir)" || {
    warn "Gatebox Rust não encontrado: precisa de ./../gatebox/gatebox-rust com Cargo.toml, ou GATEBOX_RUST_DIR. Se money/gatebox/gatebox-rust for só a pasta db/ do Docker, apague money/gatebox e volte a correr setup-env.sh (symlink → ../gatebox)."
    return 1
  }
  gb_load_env
  log "building gatebox-rust (cargo release)"
  (
    cd "${root}"
    cargo build --release
  )
}

gb_start_all() {
  local root curl_ok=0 bin

  root="$(gb_gatebox_rust_dir 2>/dev/null)" || true
  [[ -n "${root:-}" ]] || {
    warn "Gatebox Rust não encontrado — ignorando start (APICash pode usar rail simulado)"
    return 0
  }
  gb_maybe_start_infra_gatebox || true

  command -v cargo >/dev/null 2>&1 || {
    warn "Rust/cargo ausente — não é possível iniciar Gatebox"
    return 1
  }
  command -v curl >/dev/null 2>&1 && curl_ok=1

  # Sobe o simulador Sulcred antes do Gatebox (Gatebox precisa dele para gerar QR PIX)
  sulcred_start || true

  gb_stop_all || true

  bin="${root}/target/release/gatebox-rust"
  if [[ "${SKIP_BUILD:-0}" != "1" ]] || [[ ! -x "${bin}" ]]; then
    (
      cd "${root}"
      cargo build --release
    ) || {
      warn "cargo build gatebox-rust falhou"
      return 1
    }
    bin="${root}/target/release/gatebox-rust"
  fi

  [[ -x "${bin}" ]] || {
    warn "binário não encontrado após build: ${bin}"
    return 1
  }

  gb_load_env
  mkdir -p "${GB_LOG}"

  log "starting gatebox-rust (PORT=${GB_API_PORT}, logs ${GB_LOG}/gatebox-rust.log)"

  (
    cd "${root}" || exit 1
    set -a
    [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
    set +a
    export MONEY_LAN_HOST="${MONEY_LAN_HOST:-192.168.0.10}"
    local gb_p="${GB_API_PORT:-8081}"
    local gbm="${GB_METRICS_PORT:-2112}"
    local pguser="${POSTGRES_USER:-apicash}"
    local pgpass="${POSTGRES_PASSWORD:-apicash}"
    local pgport="${POSTGRES_PORT:-5432}"
    local gdb="${GATEBOX_POSTGRES_DB:-dubai-cash}"
    export PORT="${gb_p}"
    export METRICS_PORT="${gbm}"
    # Broker único por defeito (`PULSAR_URL` do money/.env, ex. 6650). Override opcional: GATEBOX_PULSAR_CLIENT_URL.
    if [[ -n "${GATEBOX_PULSAR_CLIENT_URL:-}" ]]; then
      export PULSAR_URL="${GATEBOX_PULSAR_CLIENT_URL}"
    else
      export PULSAR_URL="${PULSAR_URL:-pulsar://${MONEY_LAN_HOST}:${PULSAR_BROKER_PORT:-6650}}"
    fi
    export POSTGRESQL_WRITE_URL="${POSTGRESQL_WRITE_URL:-postgres://${pguser}:${pgpass}@${MONEY_LAN_HOST}:${pgport}/${gdb}?sslmode=disable}"
    export POSTGRESQL_READ_URL="${POSTGRESQL_READ_URL:-${POSTGRESQL_WRITE_URL}}"
    export APICASH_WHATSAPP_URL="${APICASH_WHATSAPP_URL:-http://${MONEY_LAN_HOST}:3010}"
    export APICASH_API_KEY="${APICASH_API_KEY:-}"
    exec "${bin}"
  ) >>"${GB_LOG}/gatebox-rust.log" 2>&1 &

  printf '%s\n' $! >"${GB_PID}"
  sleep 0.75
  [[ "${curl_ok}" = 1 ]] || {
    log "Gatebox iniciado sem curl para health-check"
    return 0
  }
  gb_wait_http_ok "gatebox-rust /health" "http://${RUNAPP_LOOPBACK:-127.0.0.1}:${GB_API_PORT}/health" "${GB_HEALTH_WAIT_SECONDS:-180}"
}

gb_print_status() {
  local code url
  gb_load_env
  url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${GB_API_PORT:-8081}/health"
  printf '\n== Gatebox Rust (PIX) — PORT=%s ==\n' "${GB_API_PORT:-8081}"
  if ! gb_gatebox_rust_dir >/dev/null 2>&1; then
    printf '  (diretório gatebox-rust não encontrado — veja GATEBOX_RUST_DIR / setup-env.sh)\n'
    return 0
  fi
  if command -v curl >/dev/null 2>&1; then
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "${url}" 2>/dev/null || true)"
    printf '  http health=%s  url=%s  pid=%s  log=%s/gatebox-rust.log\n' "${code:-000}" "${url}" "${GB_PID}" "${GB_LOG}"
    printf '  tail -f %s/gatebox-rust.log\n' "${GB_LOG}"
  else
    printf '  url=%s  (curl ausente)\n' "${url}"
  fi
}

gb_logs_all() {
  mkdir -p "${GB_LOG}"
  log "following Gatebox Rust log; Ctrl+C to stop"
  tail -n 80 -F "${GB_LOG}/gatebox-rust.log" 2>/dev/null || touch "${GB_LOG}/gatebox-rust.log"
}

# -----------------------------------------------------------------------------
# Sulcred — simulador do gateway PIX (porta 7020)
# Necessário para que o Gatebox gere QR Codes PIX reais no rail `simulated`.
# -----------------------------------------------------------------------------

SIM_LOG="${MONEY}/.runapp/sulcred"
SIM_PID="${SIM_LOG}/sulcred.pid"
mkdir -p "${SIM_LOG}"

sulcred_dir() {
  local candidates=(
    "${MONEY}/../gatebox/simulador_rust/sulcred"
    "${MONEY}/gatebox/simulador_rust/sulcred"
    "/home/devel/git/pos-nearx/gatebox/simulador_rust/sulcred"
  )
  local d
  for d in "${candidates[@]}"; do
    if [ -f "${d}/Cargo.toml" ]; then
      (cd "${d}" && pwd)
      return 0
    fi
  done
  return 1
}

sulcred_stop() {
  log "stopping sulcred simulator"
  if [ -f "${SIM_PID}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${SIM_PID}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" 2>/dev/null || true
      sleep 0.5
    fi
    rm -f "${SIM_PID}"
  fi
  pkill -f "target/release/sulcred" 2>/dev/null || true
  pkill -f "target/debug/sulcred"   2>/dev/null || true
}

sulcred_start() {
  local root ws_root
  root="$(sulcred_dir 2>/dev/null)" || {
    warn "Sulcred simulator não encontrado — Gatebox não conseguirá gerar QR PIX (rail simulated)"
    return 0
  }
  # simulador_rust é um workspace; o binário vai para workspace/target, não para crate/target
  ws_root="$(dirname "${root}")"

  command -v cargo >/dev/null 2>&1 || {
    warn "cargo ausente — não é possível iniciar Sulcred"
    return 1
  }

  sulcred_stop || true

  local bin="${ws_root}/target/release/sulcred"
  if [[ "${SKIP_BUILD:-0}" != "1" ]] || [[ ! -x "${bin}" ]]; then
    log "building sulcred simulator (cargo release)"
    (cd "${root}" && cargo build --release -p sulcred) || {
      warn "cargo build sulcred falhou"
      return 1
    }
  fi

  [[ -x "${bin}" ]] || {
    warn "binário sulcred não encontrado após build: ${bin}"
    return 1
  }

  log "starting sulcred simulator (PORT=7020, logs ${SIM_LOG}/sulcred.log)"
  (
    set -a
    [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
    set +a
    export PORT=7020
    exec "${bin}"
  ) >>"${SIM_LOG}/sulcred.log" 2>&1 &

  printf '%s\n' $! >"${SIM_PID}"
  sleep 0.5
  if kill -0 "$(cat "${SIM_PID}" 2>/dev/null)" 2>/dev/null; then
    log "Sulcred simulator OK (porta 7020)"
  else
    warn "Sulcred simulator falhou ao iniciar — ver ${SIM_LOG}/sulcred.log"
  fi
}

sulcred_print_status() {
  local port=7020
  printf '\n== Sulcred Simulator (gateway PIX mock) — PORT=%s ==\n' "${port}"
  if ! sulcred_dir >/dev/null 2>&1; then
    printf '  (diretório sulcred não encontrado)\n'
    return 0
  fi
  if command -v curl >/dev/null 2>&1; then
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "http://127.0.0.1:${port}/health" 2>/dev/null || true)"
    printf '  http health=%s  pid=%s  log=%s/sulcred.log\n' "${code:-000}" "${SIM_PID}" "${SIM_LOG}"
  else
    printf '  pid=%s  (curl ausente para health-check)\n' "${SIM_PID}"
  fi
}

# -----------------------------------------------------------------------------
# simulator_anchor_pix — simulador Etherfuse (on/off-ramp PIX ↔ BRLx, porta 8093)
# Necessário para modo testnet com APICASH_FIAT_RAIL=anchor sem Etherfuse real.
# -----------------------------------------------------------------------------

ANCHOR_SIM_PORT="${ANCHOR_SIM_PORT:-8093}"
ANCHOR_SIM_LOG="${MONEY}/.runapp/anchor-sim"
ANCHOR_SIM_PID="${ANCHOR_SIM_LOG}/anchor-sim.pid"
mkdir -p "${ANCHOR_SIM_LOG}"

anchor_sim_dir() {
  local candidates=(
    "${MONEY}/../gatebox/simulador_rust/simulator_anchor_pix"
    "${MONEY}/gatebox/simulador_rust/simulator_anchor_pix"
    "/home/devel/git/pos-nearx/gatebox/simulador_rust/simulator_anchor_pix"
  )
  local d
  for d in "${candidates[@]}"; do
    if [ -f "${d}/Cargo.toml" ]; then
      (cd "${d}" && pwd)
      return 0
    fi
  done
  return 1
}

anchor_sim_present() {
  anchor_sim_dir >/dev/null 2>&1
}

anchor_sim_stop() {
  log "stopping simulator-anchor-pix"
  if [ -f "${ANCHOR_SIM_PID}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${ANCHOR_SIM_PID}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" 2>/dev/null || true
      sleep 0.5
    fi
    rm -f "${ANCHOR_SIM_PID}"
  fi
  pkill -f "target/release/simulator-anchor-pix" 2>/dev/null || true
  pkill -f "target/debug/simulator-anchor-pix"   2>/dev/null || true
}

anchor_sim_start() {
  local root ws_root
  root="$(anchor_sim_dir 2>/dev/null)" || {
    warn "simulator-anchor-pix não encontrado — testnet sem on/off-ramp real"
    return 0
  }
  ws_root="$(dirname "${root}")"

  command -v cargo >/dev/null 2>&1 || {
    warn "cargo ausente — não é possível iniciar simulator-anchor-pix"
    return 1
  }

  anchor_sim_stop || true

  local bin="${ws_root}/target/release/simulator-anchor-pix"
  if [[ "${SKIP_BUILD:-0}" != "1" ]] || [[ ! -x "${bin}" ]]; then
    log "building simulator-anchor-pix (cargo release)"
    (cd "${root}" && cargo build --release -p simulator-anchor-pix) || {
      warn "cargo build simulator-anchor-pix falhou"
      return 1
    }
  fi

  [[ -x "${bin}" ]] || {
    warn "binário simulator-anchor-pix não encontrado após build: ${bin}"
    return 1
  }

  log "starting simulator-anchor-pix (PORT=${ANCHOR_SIM_PORT}, logs ${ANCHOR_SIM_LOG}/anchor-sim.log)"
  (
    set -a
    [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
    set +a
    export PORT="${ANCHOR_SIM_PORT}"
    exec "${bin}"
  ) >>"${ANCHOR_SIM_LOG}/anchor-sim.log" 2>&1 &

  printf '%s\n' $! >"${ANCHOR_SIM_PID}"
  sleep 0.5
  if kill -0 "$(cat "${ANCHOR_SIM_PID}" 2>/dev/null)" 2>/dev/null; then
    log "simulator-anchor-pix OK (porta ${ANCHOR_SIM_PORT})"
  else
    warn "simulator-anchor-pix falhou ao iniciar — ver ${ANCHOR_SIM_LOG}/anchor-sim.log"
  fi
}

anchor_sim_print_status() {
  printf '\n== simulator-anchor-pix (Etherfuse mock) — PORT=%s ==\n' "${ANCHOR_SIM_PORT}"
  if ! anchor_sim_present; then
    printf '  (diretório simulator_anchor_pix não encontrado)\n'
    return 0
  fi
  if command -v curl >/dev/null 2>&1; then
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "http://127.0.0.1:${ANCHOR_SIM_PORT}/health" 2>/dev/null || true)"
    printf '  http health=%s  log=%s/anchor-sim.log\n' "${code:-000}" "${ANCHOR_SIM_LOG}"
  else
    printf '  (curl ausente para health-check)\n'
  fi
}

anchor_sim_logs() {
  log "following simulator-anchor-pix log; Ctrl+C to stop"
  mkdir -p "${ANCHOR_SIM_LOG}"
  tail -n 80 -F "${ANCHOR_SIM_LOG}/anchor-sim.log" 2>/dev/null || true
}

# -----------------------------------------------------------------------------
# Banco Saczuck — backend Go (gatebox/banco/backend_banco)
# -----------------------------------------------------------------------------

bb_load_env() {
  set -a
  [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
  set +a
  export MONEY_LAN_HOST="${MONEY_LAN_HOST:-192.168.0.10}"
  export BANCO_HTTP_ADDR="${BANCO_HTTP_ADDR:-:8091}"
  local pguser="${POSTGRES_USER:-apicash}"
  local pgpass="${POSTGRES_PASSWORD:-apicash}"
  local pgport="${POSTGRES_PORT:-5432}"
  local pgdb="${BANCO_POSTGRES_DB:-banco_saczuck}"
  export BANCO_DATABASE_URL="${BANCO_DATABASE_URL:-postgresql://${pguser}:${pgpass}@${MONEY_LAN_HOST}:${pgport}/${pgdb}?sslmode=disable}"
  export BANCO_JWT_SECRET="${BANCO_JWT_SECRET:-${JWT_SECRET:-change-me}}"
  export GB_API_PORT="${GB_API_PORT:-8081}"
  export GATEBOX_BASE_URL="${GATEBOX_BASE_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"
  export GATEBOX_API_KEY="${GATEBOX_API_KEY:-sandbox-key}"
  export APICASH_WHATSAPP_URL="${APICASH_WHATSAPP_URL:-http://${MONEY_LAN_HOST}:3010}"
  export APICASH_CORE_URL="${APICASH_CORE_URL:-http://${MONEY_LAN_HOST}:${APICASH_HTTP_PORT:-3000}}"
  export BANCO_DEFAULT_SIM_BEHAVIOR="${BANCO_DEFAULT_SIM_BEHAVIOR:-manual}"
  export BANCO_ENVIRONMENT="${BANCO_ENVIRONMENT:-sandbox}"
}

bb_http_port() {
  bb_load_env
  printf '%s\n' "${BANCO_HTTP_ADDR##*:}"
}

bb_stop_all() {
  log "stopping backend_banco (host process)"
  local pf="${BB_LOG}/banco-backend.pid"
  if [ -f "${pf}" ]; then
    pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ]; then
      kill -TERM "${pid}" >/dev/null 2>&1 || true
      local _
      for _ in $(seq 1 40); do
        kill -0 "${pid}" >/dev/null 2>&1 || break
        sleep 0.25
      done
      kill -KILL "${pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${pf}"
  fi
  bb_load_env
  gb_kill_by_port "$(bb_http_port)"
  pkill -f "${BB_BIN}" 2>/dev/null || true
  pkill -f "go run.*gatebox/banco/backend_banco" 2>/dev/null || true
  pkill -f "go run.*/backend_banco/cmd/server" 2>/dev/null || true
}

bb_build() {
  bb_load_env
  mkdir -p "${BB_LOG}"
  log "building backend_banco (go build)"
  (cd "${BANCO_BE}" && go build -o "${BB_BIN}" ./cmd/server)
}

bb_start_all() {
  command -v go >/dev/null 2>&1 || {
    warn "go é necessário para backend_banco"
    return 1
  }
  command -v curl >/dev/null 2>&1 || {
    warn "curl é necessário para backend_banco"
    return 1
  }
  [ -f "${MONEY}/.env" ] || {
    warn "Falta ${MONEY}/.env — execute ${MONEY}/setup-env.sh"
    exit 1
  }
  [ -d "${BANCO_BE}" ] || {
    warn "backend_banco não encontrado em ${BANCO_BE}"
    return 1
  }
  bb_load_env
  mkdir -p "${BB_LOG}"
  bb_build || return 1
  log "starting backend_banco (logs: ${BB_LOG}/banco-backend.log)"
  (
    cd "${BANCO_BE}"
    exec "${BB_BIN}"
  ) >>"${BB_LOG}/banco-backend.log" 2>&1 &
  printf '%s\n' $! >"${BB_LOG}/banco-backend.pid"
  sleep 0.5
  local hp url
  hp="$(bb_http_port)"
  url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${hp}/health"
  gb_wait_http_ok "backend_banco /health" "${url}" 90
}

bb_print_status() {
  bb_load_env
  local hp url code
  hp="$(bb_http_port)"
  url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${hp}/health"
  printf '\n== backend_banco (Banco Saczuck API) ==\n'
  if command -v curl >/dev/null 2>&1; then
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "${url}" 2>/dev/null || true)"
    printf '  http=%s  addr=%s  log=%s/banco-backend.log\n' "${code:-000}" "${BANCO_HTTP_ADDR}" "${BB_LOG}"
  else
    printf '  addr=%s  log=%s/banco-backend.log (curl ausente)\n' "${BANCO_HTTP_ADDR}" "${BB_LOG}"
  fi
}

bb_logs_all() {
  log "following backend_banco log; Ctrl+C to stop"
  mkdir -p "${BB_LOG}"
  tail -n 80 -F "${BB_LOG}/banco-backend.log" 2>/dev/null || true
}

# -----------------------------------------------------------------------------
# Site público — Vite + React (holdy/site, porta SITE_PORT)
# -----------------------------------------------------------------------------

site_present() {
  [ -f "${SITE_ROOT}/package.json" ]
}

site_prepare_path() {
  export PATH="${HOME}/.bun/bin:${HOME}/.local/bin:/usr/local/bin:${PATH}"
}

# Caminho absoluto do npm/bun/pnpm (preferir bun — holdy/site tem bun.lock).
site_resolve_pm() {
  site_prepare_path
  local name bin
  for name in bun npm pnpm; do
    bin="$(command -v "${name}" 2>/dev/null)" || continue
    printf '%s\n' "${bin}"
    return 0
  done
  return 1
}

site_pkg_manager() {
  local bin
  bin="$(site_resolve_pm)" || return 1
  basename "${bin}"
}

site_stop_all() {
  log "stopping site (Vite dev server)"
  local pf="${SITE_LOG}/vite.pid"
  if [ -f "${pf}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ]; then
      kill -TERM "${pid}" >/dev/null 2>&1 || true
      local _
      for _ in $(seq 1 40); do
        kill -0 "${pid}" >/dev/null 2>&1 || break
        sleep 0.25
      done
      kill -KILL "${pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${pf}"
  fi
  gb_kill_by_port "${SITE_PORT}"
  pkill -f "vite.*${SITE_ROOT}" 2>/dev/null || true
}

site_ensure_deps() {
  local pm_bin pm
  site_present || return 1
  if [ -d "${SITE_ROOT}/node_modules" ]; then
    return 0
  fi
  pm_bin="$(site_resolve_pm)" || {
    warn "site: instale bun ou npm — ex.: curl -fsSL https://bun.sh/install | bash"
    return 1
  }
  pm="$(basename "${pm_bin}")"
  log "site: ${pm} install (primeira vez em ${SITE_ROOT})"
  (cd "${SITE_ROOT}" && "${pm_bin}" install)
}

site_build() {
  local pm_bin pm
  site_present || {
    warn "site não encontrado em ${SITE_ROOT}"
    return 1
  }
  site_ensure_deps || return 1
  pm_bin="$(site_resolve_pm)" || return 1
  pm="$(basename "${pm_bin}")"
  log "building site (vite build via ${pm})"
  (cd "${SITE_ROOT}" && "${pm_bin}" run build)
}

site_start_all() {
  local pm_bin pm
  pm_bin="$(site_resolve_pm)" || {
    warn "site: instale bun ou npm — ex.: curl -fsSL https://bun.sh/install | bash"
    return 1
  }
  pm="$(basename "${pm_bin}")"
  site_present || {
    warn "site não encontrado em ${SITE_ROOT} (defina SITE_DIR se necessário)"
    return 1
  }
  site_ensure_deps || return 1
  site_stop_all
  log "starting site (Vite dev via ${pm}, port ${SITE_PORT}, logs ${SITE_LOG}/vite.log)"
  (
    cd "${SITE_ROOT}"
    if [ "${pm}" = bun ]; then
      exec "${pm_bin}" run dev --host 127.0.0.1 --port "${SITE_PORT}"
    else
      exec "${pm_bin}" run dev -- --host 127.0.0.1 --port "${SITE_PORT}"
    fi
  ) >>"${SITE_LOG}/vite.log" 2>&1 &
  local pid=$!
  printf '%s\n' "${pid}" >"${SITE_LOG}/vite.pid"
  sleep 1
  if ! kill -0 "${pid}" 2>/dev/null; then
    warn "site (Vite) saiu durante o arranque; ver ${SITE_LOG}/vite.log"
    tail -n 30 "${SITE_LOG}/vite.log" 2>/dev/null || true
    return 1
  fi
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${SITE_PORT}/"
  if ! gb_wait_http_ok "site (Vite)" "${url}" 120; then
    warn "site não respondeu — ver ${SITE_LOG}/vite.log"
    return 1
  fi
  if ! truthy "${RUNAPP_DEFER_BROWSER:-0}"; then
    site_maybe_open_browser || true
  fi
}

site_print_status() {
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${SITE_PORT}/"
  printf '\n== site (holdy/site — Vite) ==\n'
  if command -v curl >/dev/null 2>&1; then
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "${url}" 2>/dev/null || true)"
    printf '  http=%s  url=%s  dir=%s  log=%s/vite.log\n' "${code:-000}" "${url}" "${SITE_ROOT}" "${SITE_LOG}"
  else
    printf '  url=%s  dir=%s  log=%s/vite.log (curl ausente)\n' "${url}" "${SITE_ROOT}" "${SITE_LOG}"
  fi
}

site_logs_all() {
  log "following site log; Ctrl+C to stop"
  mkdir -p "${SITE_LOG}"
  tail -n 80 -F "${SITE_LOG}/vite.log" 2>/dev/null || true
}

# -----------------------------------------------------------------------------
# holdfy-admin — Vite + React (painel APICash, porta HOLDFY_ADMIN_PORT)
# -----------------------------------------------------------------------------

holdfy_admin_present() {
  [ -f "${HOLDFY_ADMIN_ROOT}/package.json" ]
}

holdfy_admin_ensure_deps() {
  holdfy_admin_present || return 1
  [ -d "${HOLDFY_ADMIN_ROOT}/node_modules" ] && return 0
  local pm_bin
  pm_bin="$(site_resolve_pm)" || { warn "holdfy-admin: instale bun ou npm"; return 1; }
  log "holdfy-admin: $(basename "${pm_bin}") install"
  (cd "${HOLDFY_ADMIN_ROOT}" && "${pm_bin}" install)
}

holdfy_admin_stop() {
  log "stopping holdfy-admin"
  local pf="${HOLDFY_ADMIN_LOG}/holdfy-admin.pid"
  if [ -f "${pf}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ]; then
      kill -TERM "${pid}" >/dev/null 2>&1 || true
      local _; for _ in $(seq 1 40); do kill -0 "${pid}" >/dev/null 2>&1 || break; sleep 0.25; done
      kill -KILL "${pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${pf}"
  fi
  gb_kill_by_port "${HOLDFY_ADMIN_PORT}"
}

holdfy_admin_start() {
  holdfy_admin_present || { warn "holdfy-admin não encontrado em ${HOLDFY_ADMIN_ROOT}"; return 1; }
  holdfy_admin_ensure_deps || return 1
  holdfy_admin_stop
  local pm_bin pm
  pm_bin="$(site_resolve_pm)" || return 1
  pm="$(basename "${pm_bin}")"
  log "starting holdfy-admin (${pm}, port ${HOLDFY_ADMIN_PORT}, logs ${HOLDFY_ADMIN_LOG}/holdfy-admin.log)"
  (
    cd "${HOLDFY_ADMIN_ROOT}"
    if [ "${pm}" = bun ]; then
      exec "${pm_bin}" run dev --host 127.0.0.1 --port "${HOLDFY_ADMIN_PORT}"
    else
      exec "${pm_bin}" run dev -- --host 127.0.0.1 --port "${HOLDFY_ADMIN_PORT}"
    fi
  ) >>"${HOLDFY_ADMIN_LOG}/holdfy-admin.log" 2>&1 &
  printf '%s\n' $! >"${HOLDFY_ADMIN_LOG}/holdfy-admin.pid"
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${HOLDFY_ADMIN_PORT}/"
  gb_wait_http_ok "holdfy-admin" "${url}" 60 || warn "holdfy-admin não respondeu — ver ${HOLDFY_ADMIN_LOG}/holdfy-admin.log"
}

holdfy_admin_print_status() {
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${HOLDFY_ADMIN_PORT}/"
  printf '\n== holdfy-admin (APICash admin — Vite) ==\n'
  if command -v curl >/dev/null 2>&1; then
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "${url}" 2>/dev/null || true)"
    printf '  http=%s  url=%s  log=%s/holdfy-admin.log\n' "${code:-000}" "${url}" "${HOLDFY_ADMIN_LOG}"
  else
    printf '  url=%s  log=%s/holdfy-admin.log\n' "${url}" "${HOLDFY_ADMIN_LOG}"
  fi
}

# -----------------------------------------------------------------------------
# front-gatebox — CRA + React (painel PIX Gatebox, porta FRONT_GATEBOX_PORT)
# -----------------------------------------------------------------------------

front_gatebox_present() {
  [ -f "${FRONT_GATEBOX_ROOT}/package.json" ]
}

front_gatebox_ensure_deps() {
  front_gatebox_present || return 1
  [ -d "${FRONT_GATEBOX_ROOT}/node_modules" ] && return 0
  local pm_bin
  pm_bin="$(site_resolve_pm)" || { warn "front-gatebox: instale bun ou npm"; return 1; }
  log "front-gatebox: $(basename "${pm_bin}") install"
  (cd "${FRONT_GATEBOX_ROOT}" && "${pm_bin}" install)
}

front_gatebox_stop() {
  log "stopping front-gatebox"
  local pf="${FRONT_GATEBOX_LOG}/front-gatebox.pid"
  if [ -f "${pf}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ]; then
      kill -TERM "${pid}" >/dev/null 2>&1 || true
      local _; for _ in $(seq 1 40); do kill -0 "${pid}" >/dev/null 2>&1 || break; sleep 0.25; done
      kill -KILL "${pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${pf}"
  fi
  gb_kill_by_port "${FRONT_GATEBOX_PORT}"
}

front_gatebox_start() {
  front_gatebox_present || { warn "front-gatebox não encontrado em ${FRONT_GATEBOX_ROOT}"; return 1; }
  front_gatebox_ensure_deps || return 1
  front_gatebox_stop
  local pm_bin
  pm_bin="$(site_resolve_pm)" || { warn "front-gatebox: instale bun ou npm"; return 1; }
  log "starting front-gatebox (CRA via $(basename "${pm_bin}"), port ${FRONT_GATEBOX_PORT}, logs ${FRONT_GATEBOX_LOG}/front-gatebox.log)"
  (
    cd "${FRONT_GATEBOX_ROOT}"
    ac_load_env 2>/dev/null || true
    export PORT="${FRONT_GATEBOX_PORT}"
    export BROWSER=none
    export REACT_APP_API_BASE_URL="${REACT_APP_API_BASE_URL:-http://${RUNAPP_LOOPBACK:-127.0.0.1}:${GB_API_PORT:-8081}/api/v1}"
    exec "${pm_bin}" run start
  ) >>"${FRONT_GATEBOX_LOG}/front-gatebox.log" 2>&1 &
  printf '%s\n' $! >"${FRONT_GATEBOX_LOG}/front-gatebox.pid"
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${FRONT_GATEBOX_PORT}/"
  gb_wait_http_ok "front-gatebox" "${url}" 120 || warn "front-gatebox não respondeu — ver ${FRONT_GATEBOX_LOG}/front-gatebox.log"
}

front_gatebox_print_status() {
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${FRONT_GATEBOX_PORT}/"
  printf '\n== front-gatebox (Gatebox admin — CRA) ==\n'
  if command -v curl >/dev/null 2>&1; then
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 1 "${url}" 2>/dev/null || true)"
    printf '  http=%s  url=%s  log=%s/front-gatebox.log\n' "${code:-000}" "${url}" "${FRONT_GATEBOX_LOG}"
  else
    printf '  url=%s  log=%s/front-gatebox.log\n' "${url}" "${FRONT_GATEBOX_LOG}"
  fi
}

# -----------------------------------------------------------------------------
# scraper-service — headless browser Python/Playwright, porta SCRAPER_PORT
# -----------------------------------------------------------------------------

scraper_present() {
  [ -f "${SCRAPER_ROOT}/scraper.py" ]
}

scraper_python() {
  # Prefere o venv local; cai para python3 do sistema se não existir ainda
  if [ -x "${SCRAPER_ROOT}/.venv/bin/python" ]; then
    printf '%s' "${SCRAPER_ROOT}/.venv/bin/python"
  else
    printf '%s' "$(command -v python3 2>/dev/null || echo python3)"
  fi
}

scraper_ensure_deps() {
  scraper_present || return 1

  local py
  py="$(command -v python3 2>/dev/null || true)"
  if [ -z "${py}" ]; then
    warn "scraper-service: python3 não encontrado — instale com: sudo apt install python3 python3-venv"
    return 1
  fi

  # Criar venv se não existir
  if [ ! -x "${SCRAPER_ROOT}/.venv/bin/python" ]; then
    log "scraper-service: criando venv Python"
    "${py}" -m venv "${SCRAPER_ROOT}/.venv" || { warn "scraper-service: falha ao criar venv"; return 1; }
  fi

  local pip="${SCRAPER_ROOT}/.venv/bin/pip"

  # Instalar dependências Python
  if ! "${SCRAPER_ROOT}/.venv/bin/python" -c "import playwright" 2>/dev/null; then
    log "scraper-service: instalando dependências Python (playwright, aiohttp)"
    "${pip}" install -q -r "${SCRAPER_ROOT}/requirements.txt" || { warn "scraper-service: pip install falhou"; return 1; }
  fi

  # Instalar Chromium do Playwright
  if ! "${SCRAPER_ROOT}/.venv/bin/python" -m playwright install --dry-run chromium 2>/dev/null | grep -q "Chromium"; then
    log "scraper-service: instalando Chromium (playwright)"
    "${SCRAPER_ROOT}/.venv/bin/python" -m playwright install chromium || \
      warn "scraper-service: falha ao instalar Chromium — tente: cd ${SCRAPER_ROOT} && .venv/bin/python -m playwright install chromium"
  fi
}

scraper_stop() {
  local pf="${SCRAPER_LOG}/scraper.pid"
  if [ -f "${pf}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ]; then
      kill -TERM "${pid}" >/dev/null 2>&1 || true
      local _i; for _i in $(seq 1 40); do kill -0 "${pid}" >/dev/null 2>&1 || break; sleep 0.25; done
      kill -KILL "${pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${pf}"
  fi
  gb_kill_by_port "${SCRAPER_PORT}"
}

scraper_start() {
  scraper_present || { warn "scraper-service não encontrado em ${SCRAPER_ROOT}"; return 1; }
  scraper_ensure_deps || return 1
  scraper_stop

  local py
  py="$(scraper_python)"

  set -a; [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"; set +a
  export SCRAPER_PORT="${SCRAPER_PORT:-4000}"
  export SCRAPER_API_KEY="${SCRAPER_API_KEY:-${APICASH_API_KEY:-}}"

  log "starting scraper-service (Python/Playwright, port ${SCRAPER_PORT}, logs ${SCRAPER_LOG}/scraper.log)"
  (
    cd "${SCRAPER_ROOT}"
    exec "${py}" scraper.py server
  ) >>"${SCRAPER_LOG}/scraper.log" 2>&1 &
  printf '%s\n' $! >"${SCRAPER_LOG}/scraper.pid"

  gb_wait_http_ok "scraper-service" "http://${RUNAPP_LOOPBACK:-127.0.0.1}:${SCRAPER_PORT}/health" 30 || \
    warn "scraper-service não respondeu — ver ${SCRAPER_LOG}/scraper.log"
}

scraper_print_status() {
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${SCRAPER_PORT}/health"
  printf '\n== scraper-service (Python/Playwright) — port %s ==\n' "${SCRAPER_PORT}"
  if command -v curl >/dev/null 2>&1; then
    local body code
    code="$(curl -sS -o /tmp/_scraper_health.json -w '%{http_code}' --max-time 2 "${url}" 2>/dev/null || echo 000)"
    if [ "${code}" = "200" ] && [ -f /tmp/_scraper_health.json ]; then
      local session
      session="$(python3 -c "import json,sys; d=json.load(open('/tmp/_scraper_health.json')); print(d.get('session','?'))" 2>/dev/null || echo "?")"
      printf '  http=%s  session=%s  url=%s\n' "${code}" "${session}" "${url}"
      if [ "${session}" = "missing" ]; then
        printf '  AVISO: sem cookies — execute: cd %s && .venv/bin/python scraper.py login\n' "${SCRAPER_ROOT}"
      fi
    else
      printf '  http=%s  log=%s/scraper.log\n' "${code:-000}" "${SCRAPER_LOG}"
    fi
  fi
}

# -----------------------------------------------------------------------------
# apprastreio/backend — simulador de rastreio Rust, porta RASTREIO_PORT (8092)
# -----------------------------------------------------------------------------

rastreio_present() {
  [ -f "${RASTREIO_ROOT}/Cargo.toml" ]
}

rastreio_build() {
  rastreio_present || { warn "apprastreio/backend não encontrado em ${RASTREIO_ROOT}"; return 1; }
  log "building logistica-holdfy-backend (cargo release)"
  (cd "${RASTREIO_ROOT}" && cargo build --release)
}

rastreio_stop() {
  local pf="${RASTREIO_LOG}/rastreio.pid"
  if [ -f "${pf}" ]; then
    local pid
    pid="$(tr -d '[:space:]' <"${pf}" 2>/dev/null || true)"
    if [ -n "${pid:-}" ]; then
      kill -TERM "${pid}" >/dev/null 2>&1 || true
      local _i; for _i in $(seq 1 40); do kill -0 "${pid}" >/dev/null 2>&1 || break; sleep 0.25; done
      kill -KILL "${pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${pf}"
  fi
  gb_kill_by_port "${RASTREIO_PORT}"
  pkill -f "logistica-holdfy-backend" 2>/dev/null || true
}

rastreio_start() {
  rastreio_present || { warn "apprastreio/backend não encontrado em ${RASTREIO_ROOT}"; return 1; }
  rastreio_stop
  rastreio_build || { warn "build logistica-holdfy-backend falhou"; return 1; }

  local bin="${RASTREIO_ROOT}/target/release/logistica-holdfy-backend"

  set -a; [ -f "${MONEY}/.env" ] && . "${MONEY}/.env"; set +a
  export LOGISTICA_HTTP_PORT="${RASTREIO_PORT}"

  log "starting logistica-holdfy-backend (port ${RASTREIO_PORT}, logs ${RASTREIO_LOG}/rastreio.log)"
  (
    cd "${RASTREIO_ROOT}"
    exec "${bin}"
  ) >>"${RASTREIO_LOG}/rastreio.log" 2>&1 &
  printf '%s\n' $! >"${RASTREIO_LOG}/rastreio.pid"

  gb_wait_http_ok "logistica-holdfy-backend /health" \
    "http://${RUNAPP_LOOPBACK:-127.0.0.1}:${RASTREIO_PORT}/health" 60 || \
    warn "logistica-holdfy-backend não respondeu — ver ${RASTREIO_LOG}/rastreio.log"
}

rastreio_print_status() {
  local url="http://${RUNAPP_LOOPBACK:-127.0.0.1}:${RASTREIO_PORT}/health"
  printf '\n== logistica-holdfy-backend (rastreio) — port %s ==\n' "${RASTREIO_PORT}"
  if command -v curl >/dev/null 2>&1; then
    local code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --max-time 2 "${url}" 2>/dev/null || echo 000)"
    printf '  http=%s  url=%s  log=%s/rastreio.log\n' "${code:-000}" "${url}" "${RASTREIO_LOG}"
  else
    printf '  log=%s/rastreio.log (curl ausente)\n' "${RASTREIO_LOG}"
  fi
}

rastreio_logs() {
  log "following rastreio log; Ctrl+C to stop"
  mkdir -p "${RASTREIO_LOG}"
  tail -n 80 -F "${RASTREIO_LOG}/rastreio.log" 2>/dev/null || true
}

print_infra_hint() {
  if command -v docker >/dev/null 2>&1; then
    printf '\n== Docker (money/docker-compose.yml) ==\n'
    compose_money ps 2>/dev/null || true
  fi
}

usage() {
  cat <<USAGE
Usage: ${MONEY}/runapp.sh [command] [scope]

Commands:
  restart (default)  stop <scope> + start <scope> (sem argumentos: stop all + start all)
  start              Sobe backends (scope all/apicash: cargo build antes de subir)
  stop               Para processos no host (scope all: site + banco + gatebox + APICash)
  status             Estado das apps + docker compose ps
  build              cargo build conforme scope (APICash ou gatebox)
  logs [scope]       tail -F dos logs
  whatsapp-pair      Apaga sessão WhatsApp, recria pasta do QR e abre pair.html no browser
  open-browsers      Abre QR WhatsApp + site (2 abas) — útil para testar sem restart completo
  testnet            Para tudo, compila com --features soroban e sobe em modo Stellar testnet
                       (SOROBAN_ENABLED=1, REQUIRE_TESTNET=1, STELLAR_NETWORK=testnet)
  mainnet            Para tudo, compila com --features soroban e sobe em modo Stellar mainnet
                       (SOROBAN_ENABLED=1, STELLAR_NETWORK=mainnet)

Scope (opcional):
  all       build APICash; se gatebox/gatebox-rust existir (ou GATEBOX_RUST_DIR), opcionalmente sobe Gatebox + APICash
  apicash   só APICash (core, admin-backend, frontend Leptos :3002, whatsapp)
  gatebox   API Rust Gatebox (PIX) — infra: ./runinfra.sh (Postgres Gatebox; mesmo Pulsar/Redis que APICash)
  banco     apenas API Go em gatebox/banco/backend_banco (se existir)
  site      apenas site público Vite (holdy/site), defeito :5173
  rastreio  apenas logistica-holdfy-backend (simulador de rastreio Rust, porta RASTREIO_PORT :8092)
  anchor-sim simulador Etherfuse (on/off-ramp PIX ↔ BRLx testnet), porta ANCHOR_SIM_PORT :8093
  testnet   (só para logs) — filtra Stellar/BRLx/Soroban/settle/erros em tempo real

Env útil:
  RUNAPP_AUTO_SITE=1 (defeito) — com stack all, iniciar também holdy/site (npm run dev)
  SITE_DIR=/abs/holdy/site — override do caminho do site
  SITE_PORT — porta do Vite (defeito 5173; evita conflito com Pulsar :8080)
  RUNAPP_OPEN_SITE_BROWSER=1 (defeito) — abre http://127.0.0.1:5173/ após subir o site (como o QR WhatsApp)
  APICASH_WA_OPEN_BROWSER=1 (defeito) — abre file://…/whatsapp_qrcode/pair.html
  RUNAPP_LOOPBACK=127.0.0.1 (defeito) — host para health checks locais (runapp); MONEY_LAN_HOST continua para URLs na LAN
  RUNAPP_AUTO_GATEBOX=1 (defeito) — com stack all, iniciar também gatebox-rust antes do APICash; RUNAPP_AUTO_GATEBOX=0 desliga
  APICASH_FRONTEND_PORT — dashboard HoldFy (Leptos SSR), defeito 3002
  APICASH_GATEBOX_ENABLED — defeito 1 no runapp (PIX EMV via Gatebox); 0 = desativa Gatebox (rail simulado falha sem PIX)
  GATEBOX_BASE_URL — defeito http://\$MONEY_LAN_HOST:\$GB_API_PORT (MONEY_LAN_HOST defeito 192.168.86.64)
  GATEBOX_RUST_DIR=/c/abs/gatebox-rust — override do caminho quando não usa symlink money/gatebox
  BANCO_BACKEND_DIR=/abs/gatebox/banco/backend_banco — se o repo gatebox não está em money/gatebox
  GATEBOX_PULSAR_CLIENT_URL=… — só se precisar de broker distinto do PULSAR_URL global (caso excecional)
  UNIFIED_SKIP_GATEBOX_INFRA=0   — faz ./gatebox/gatebox-rust/scripts/start-infra.sh antes da API (normalmente dispensável com compose money/)

Fluxo típico:
  ${MONEY}/setup-env.sh     # cria/atualiza money/.env + symlinks + infra Docker
  ${MONEY}/runapp.sh        # stop all + start all (recomendado após mudar código)
  ${MONEY}/runapp.sh start  # só sobe (sem parar antes)
USAGE
}

CMD="${1:-}"
SCOPE="${2:-all}"

if [ -z "${CMD}" ]; then
  printf '\nEscolha o modo de rede:\n  1) testnet\n  2) mainnet\n\n> '
  read -r _choice
  case "${_choice}" in
  1 | testnet) CMD="testnet" ;;
  2 | mainnet) CMD="mainnet" ;;
  *)
    warn "opção inválida: ${_choice}"
    exit 2
    ;;
  esac
fi

if [ "${CMD}" != "whatsapp-pair" ] && [ "${CMD}" != "open-browsers" ]; then
  case "${SCOPE}" in
  all | apicash | gatebox | banco | site | sulcred | rastreio | anchor-sim | testnet) ;;
  *)
    warn "scope inválido: ${SCOPE}"
    usage
    exit 2
    ;;
  esac
fi

run_stop() {
  case "${SCOPE}" in
  all)
    rastreio_stop
    anchor_sim_stop
    bb_stop_all
    scraper_stop
    front_gatebox_stop
    holdfy_admin_stop
    site_stop_all
    gb_stop_all
    sulcred_stop
    ac_stop_all
    ;;
  apicash) ac_stop_all ;;
  gatebox) gb_stop_all; sulcred_stop ;;
  sulcred) sulcred_stop ;;
  banco) bb_stop_all ;;
  site) site_stop_all ;;
  rastreio) rastreio_stop ;;
  anchor-sim) anchor_sim_stop ;;
  esac
}

run_build() {
  case "${SCOPE}" in
  all)
    ac_build_all
    if gb_gatebox_rust_dir >/dev/null 2>&1; then
      gb_build || return 1
    fi
    if site_present; then
      site_ensure_deps || true
    fi
    ;;
  apicash) ac_build_all ;;
  gatebox) gb_build || return 1 ;;
  banco) bb_build ;;
  site) site_build ;;
  rastreio) rastreio_build || return 1 ;;
  esac
}

run_start() {
  case "${SCOPE}" in
  all)
    export RUNAPP_DEFER_BROWSER=1
    ac_build_all
    if truthy "${RUNAPP_AUTO_GATEBOX:-1}"; then
      if gb_gatebox_rust_dir >/dev/null 2>&1; then
        gb_start_all || return 1
      fi
    fi
    ac_start_all
    if truthy "${RUNAPP_AUTO_SITE:-1}" && site_present; then
      site_start_all || warn "site (holdy/site) não subiu — ver ${SITE_LOG}/vite.log"
    fi
    if holdfy_admin_present; then
      holdfy_admin_start || warn "holdfy-admin não subiu — ver ${HOLDFY_ADMIN_LOG}/holdfy-admin.log"
    fi
    if front_gatebox_present; then
      front_gatebox_start || warn "front-gatebox não subiu — ver ${FRONT_GATEBOX_LOG}/front-gatebox.log"
    fi
    if scraper_present; then
      scraper_start || warn "scraper-service não subiu — ver ${SCRAPER_LOG}/scraper.log"
    fi
    if anchor_sim_present; then
      anchor_sim_start || warn "simulator-anchor-pix não subiu — ver ${ANCHOR_SIM_LOG}/anchor-sim.log"
    fi
    if [ -d "${BANCO_BE}" ]; then
      bb_start_all || warn "backend_banco não subiu — app Flutter (:8091) falha com connection refused"
    fi
    if rastreio_present; then
      rastreio_start || warn "logistica-holdfy-backend não subiu — ver ${RASTREIO_LOG}/rastreio.log"
    fi
    runapp_open_startup_browsers || true
    unset RUNAPP_DEFER_BROWSER
    ;;
  apicash)
    ac_build_all
    ac_start_all
    ;;
  gatebox)
    sulcred_start || true
    gb_start_all || return 1
    ;;
  sulcred)
    sulcred_start || return 1
    ;;
  banco)
    bb_start_all || return 1
    ;;
  site)
    site_start_all || return 1
    ;;
  rastreio)
    rastreio_start || return 1
    ;;
  anchor-sim)
    anchor_sim_start || return 1
    ;;
  esac
}

run_status() {
  case "${SCOPE}" in
  all | apicash) ac_print_status ;;
  esac
  case "${SCOPE}" in
  all | gatebox | sulcred) sulcred_print_status ;;
  esac
  case "${SCOPE}" in
  all | gatebox) gb_print_status ;;
  esac
  case "${SCOPE}" in
  all | banco) bb_print_status ;;
  esac
  case "${SCOPE}" in
  all | site) site_print_status ;;
  esac
  case "${SCOPE}" in
  all | rastreio) rastreio_print_status ;;
  esac
  case "${SCOPE}" in
  all | anchor-sim) anchor_sim_print_status ;;
  esac
  case "${SCOPE}" in
  all)
    holdfy_admin_print_status
    front_gatebox_print_status
    scraper_print_status
    print_infra_hint
    ;;
  esac
}

logs_testnet() {
  # Todos os logs relevantes para transações Stellar/testnet filtrados numa única stream.
  local files=()
  [ -f "${AC_LOG}/apicash-core.log" ]    && files+=("${AC_LOG}/apicash-core.log")
  [ -f "${AC_LOG}/apicash-whatsapp.log" ] && files+=("${AC_LOG}/apicash-whatsapp.log")
  [ -f "${ANCHOR_SIM_LOG}/anchor-sim.log" ] && files+=("${ANCHOR_SIM_LOG}/anchor-sim.log")

  # Cria os ficheiros se ainda não existirem para o tail não falhar
  for f in "${AC_LOG}/apicash-core.log" "${AC_LOG}/apicash-whatsapp.log" "${ANCHOR_SIM_LOG}/anchor-sim.log"; do
    [ -f "${f}" ] || touch "${f}" 2>/dev/null || true
    files+=("${f}")
  done
  # Remove duplicados mantendo ordem
  local -A seen; local uniq=()
  for f in "${files[@]}"; do
    [[ -n "${seen[$f]:-}" ]] && continue
    seen[$f]=1; uniq+=("$f")
  done

  log "=== HoldFy Testnet — transações Stellar, settle, BRLx, Soroban, erros ==="
  log "Ficheiros: ${uniq[*]}"
  log "Ctrl+C para sair"
  printf '\n'

  # Filtra linhas relevantes: Stellar, BRLx, Soroban, settle, anchor, erros e warns
  tail -n 80 -F "${uniq[@]}" 2>/dev/null \
    | grep --line-buffered -i \
        -e "stellar\|soroban\|brlx\|escrow\|testnet" \
        -e "settle\|lock_funds\|on.ramp\|off.ramp\|anchor" \
        -e "invoke_contract\|transfer_brlx\|issue_brlx" \
        -e "simulator.anchor\|deposit\|withdraw\|pix_key" \
        -e " ERROR \| WARN \|error =\|warn =" \
        -e "BRLx\|bloqueado\|liberado\|release\|custody"
}

run_logs() {
  case "${SCOPE}" in
  all)
    warn "logs all: use logs apicash | logs gatebox | logs banco | logs site | logs rastreio | logs testnet em terminais separados"
    ac_logs_all
    ;;
  apicash) ac_logs_all ;;
  gatebox) gb_logs_all ;;
  banco) bb_logs_all ;;
  site) site_logs_all ;;
  rastreio) rastreio_logs ;;
  anchor-sim) anchor_sim_logs ;;
  testnet) logs_testnet ;;
  esac
}

case "${CMD}" in
testnet | mainnet)
  RUNAPP_NETWORK_MODE="${CMD}"
  ac_setup_whatsapp_qr_dir || true
  log "modo ${CMD}: stop + build com soroban + start ${SCOPE}"
  run_stop
  run_start
  run_status
  if [ "${CMD}" = "testnet" ]; then
    logs_testnet
  fi
  ;;
open-browsers)
  ac_setup_whatsapp_qr_dir || true
  runapp_open_startup_browsers || true
  ;;
whatsapp-pair)
  ac_prepare_runtime_env
  ac_whatsapp_pair_reset
  ;;
restart)
  ac_setup_whatsapp_qr_dir || true
  log "restart ${SCOPE}: stop then start"
  run_stop
  run_start
  run_status
  ;;
start)
  ac_setup_whatsapp_qr_dir || true
  run_start
  run_status
  ;;
stop)
  run_stop
  run_status
  ;;
status)
  run_status
  ;;
build)
  run_build
  ;;
logs)
  run_logs
  ;;
-h | --help | help)
  usage
  ;;
*)
  usage
  exit 2
  ;;
esac
