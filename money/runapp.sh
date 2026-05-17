#!/usr/bin/env bash
#
# Arranque das apps do workspace: APICash, Gatebox Rust (PIX), opcionalmente backend_banco Go.
# Gatebox: gatebox/gatebox-rust (symlink criado por setup-env.sh para ../gatebox se existir) ou GATEBOX_RUST_DIR.
# Infra Docker: money/docker-compose.yml (postgres APICash + gatebox-postgres, Redis/Pulsar únicos).
#
# Integração WhatsApp (B) → apicash-core → âncora → Gatebox PIX EMV:
#   · APICash: GATEBOX_BASE_URL, APICASH_GATEBOX_ENABLED; gateways PIX (Sulcred/etc.) só no Gatebox — ver gatebox-rust + money/.env partilhado
#   · `./runapp.sh start all`: sobe gatebox-rust antes do APICash se RUNAPP_AUTO_GATEBOX=1 (não arranca simuladores de gateway)
#   · Sem Gatebox ativo + GATEBOX_BASE_URL, o rail simulado não obtém PIX (pedidos falham ao criar instrução).
#
# Uso:
#   ./runapp.sh                    # = stop all + start all (scope all, default)
#   ./runapp.sh restart [scope]    # idem: para tudo no scope e sobe de novo (+ build no start)
#   ./runapp.sh stop [apicash|gatebox|banco|all]   (stop apicash não para o Gatebox)
#   ./runapp.sh start [apicash|gatebox|banco|all]
#   ./runapp.sh status
#   ./runapp.sh build [apicash|banco|all]
#   ./runapp.sh logs [apicash|gatebox|banco|all]
#
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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

# WhatsApp pareamento: pasta do PNG + pair.html para o browser. Sobrescrever com WA_QR_DIR no .env
WA_QR_DIR_DEFAULT="${WA_QR_DIR_DEFAULT:-/home/devel/git/pos-nearx/whatsapp_qrcode}"

# Gatebox ports (prefixo GB_* para não colidir com API_PORT do .env do APICash)
GB_API_PORT="${GB_API_PORT:-8081}"
GB_METRICS_PORT="${GB_METRICS_PORT:-2112}"
CLIENT_SIM_PORT="${CLIENT_SIM_PORT:-7070}"
SEVENTRUST_PORT="${SEVENTRUST_PORT:-7010}"
GATEWAY_URL="${GATEWAY_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"
WEBHOOK_URL="${WEBHOOK_URL:-http://${MONEY_LAN_HOST}:${GB_API_PORT}}"

# Compose money: um único Pulsar (serviço `pulsar`); gatebox-postgres à parte. scripts/start-infra.sh só se UNIFIED_SKIP_GATEBOX_INFRA=0.
UNIFIED_SKIP_GATEBOX_INFRA="${UNIFIED_SKIP_GATEBOX_INFRA:-1}"

log() { printf '[money/runapp] %s\n' "$*"; }
warn() { printf '[money/runapp][warn] %s\n' "$*" >&2; }

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
  WA_QR_DIR="${WA_QR_DIR:-${WA_QR_DIR_DEFAULT:-/home/devel/git/pos-nearx/whatsapp_qrcode}}"
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

ac_prepare_runtime_env() {
  ac_load_env

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
  cargo build -p apicash-admin-backend
  cargo build -p apicash-frontend --features ssr
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
  ac_maybe_open_whatsapp_pairing_browser || true
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

ac_maybe_open_whatsapp_pairing_browser() {
  ac_load_env
  case "${APICASH_WA_TRANSPORT:-rust}" in
  cloud) return 0 ;;
  esac
  if ! truthy "${APICASH_WA_OPEN_BROWSER:-1}"; then
    return 0
  fi
  if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    warn "sem DISPLAY/WAYLAND — abra manualmente: ${WA_QR_DIR}/pair.html (ou o PNG em ${APICASH_WA_QR_PNG})"
    return 0
  fi
  ac_setup_whatsapp_qr_dir || return 0
  local html
  html="$(cd "${WA_QR_DIR}" && pwd)/pair.html"
  local browser_png="${WA_QR_DIR}/whatsapp-pairing-qr.png"
  local waited=0
  while [ ! -s "${browser_png}" ] && [ ! -s "${APICASH_WA_QR_PNG}" ] && [ "${waited}" -lt 90 ]; do
    sleep 1
    waited=$((waited + 1))
    if [ -s "${APICASH_WA_QR_PNG}" ] && [ "${APICASH_WA_QR_PNG}" != "${browser_png}" ]; then
      cp -f "${APICASH_WA_QR_PNG}" "${browser_png}" 2>/dev/null || true
    fi
  done
  if [ ! -s "${browser_png}" ]; then
    warn "QR ainda não gerado (sessão já pareada?). Para novo QR: ./runapp.sh whatsapp-pair"
    warn "Ou abra quando aparecer: file://${html}"
  fi
  if command -v xdg-open >/dev/null 2>&1; then
    log "abrindo pareamento WhatsApp no browser: file://${html}"
    xdg-open "file://${html}" >/dev/null 2>&1 &
  elif command -v gio >/dev/null 2>&1; then
    log "abrindo pareamento WhatsApp (gio): file://${html}"
    gio open "file://${html}" >/dev/null 2>&1 &
  else
    warn "xdg-open/gio ausente — abra no browser: file://${html}"
  fi
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
  log "UNIFIED_SKIP_GATEBOX_INFRA=0 — executing ${scripts} (compose money: Postgres Gatebox + Pulsar único)"
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
    local guser="${GATEBOX_POSTGRES_USER:-postgres}"
    local gpw="${GATEBOX_POSTGRES_PASSWORD:-root}"
    local gport="${GATEBOX_POSTGRES_PORT:-5433}"
    local gdb="${GATEBOX_POSTGRES_DB:-dubai-cash}"
    export PORT="${gb_p}"
    export METRICS_PORT="${gbm}"
    # Broker único por defeito (`PULSAR_URL` do money/.env, ex. 6650). Override opcional: GATEBOX_PULSAR_CLIENT_URL.
    if [[ -n "${GATEBOX_PULSAR_CLIENT_URL:-}" ]]; then
      export PULSAR_URL="${GATEBOX_PULSAR_CLIENT_URL}"
    else
      export PULSAR_URL="${PULSAR_URL:-pulsar://${MONEY_LAN_HOST}:${PULSAR_BROKER_PORT:-6650}}"
    fi
    export POSTGRESQL_WRITE_URL="${POSTGRESQL_WRITE_URL:-postgres://${guser}:${gpw}@${MONEY_LAN_HOST}:${gport}/${gdb}?sslmode=disable}"
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
  stop               Para processos no host (scope all: banco + gatebox + APICash)
  status             Estado das apps + docker compose ps
  build              cargo build conforme scope (APICash ou gatebox)
  logs [scope]       tail -F dos logs
  whatsapp-pair      Apaga sessão WhatsApp, recria pasta do QR e abre pair.html no browser

Scope (opcional):
  all       build APICash; se gatebox/gatebox-rust existir (ou GATEBOX_RUST_DIR), opcionalmente sobe Gatebox + APICash
  apicash   só APICash
  gatebox   API Rust Gatebox (PIX) — infra: ./runinfra.sh (Postgres Gatebox; mesmo Pulsar/Redis que APICash)
  banco     apenas API Go em gatebox/banco/backend_banco (se existir)

Env útil:
  RUNAPP_LOOPBACK=127.0.0.1 (defeito) — host para health checks locais (runapp); MONEY_LAN_HOST continua para URLs na LAN
  RUNAPP_AUTO_GATEBOX=1 (defeito) — com stack all, iniciar também gatebox-rust antes do APICash; RUNAPP_AUTO_GATEBOX=0 desliga
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

CMD="${1:-restart}"
SCOPE="${2:-all}"

if [ "${CMD}" != "whatsapp-pair" ]; then
  case "${SCOPE}" in
  all | apicash | gatebox | banco) ;;
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
    bb_stop_all
    gb_stop_all
    ac_stop_all
    ;;
  apicash) ac_stop_all ;;
  gatebox) gb_stop_all ;;
  banco) bb_stop_all ;;
  esac
}

run_build() {
  case "${SCOPE}" in
  all)
    ac_build_all
    if gb_gatebox_rust_dir >/dev/null 2>&1; then
      gb_build || return 1
    fi
    ;;
  apicash) ac_build_all ;;
  gatebox) gb_build || return 1 ;;
  banco) bb_build ;;
  esac
}

run_start() {
  case "${SCOPE}" in
  all)
    ac_build_all
    if truthy "${RUNAPP_AUTO_GATEBOX:-1}"; then
      if gb_gatebox_rust_dir >/dev/null 2>&1; then
        gb_start_all || return 1
      fi
    fi
    ac_start_all
    if [ -d "${BANCO_BE}" ]; then
      bb_start_all || warn "backend_banco não subiu — app Flutter (:8091) falha com connection refused"
    fi
    ;;
  apicash)
    ac_build_all
    ac_start_all
    ;;
  gatebox)
    gb_start_all || return 1
    ;;
  banco)
    bb_start_all || return 1
    ;;
  esac
}

run_status() {
  case "${SCOPE}" in
  all | apicash) ac_print_status ;;
  esac
  case "${SCOPE}" in
  all | gatebox) gb_print_status ;;
  esac
  case "${SCOPE}" in
  all | banco) bb_print_status ;;
  esac
  case "${SCOPE}" in
  all)
    print_infra_hint
    ;;
  esac
}

run_logs() {
  case "${SCOPE}" in
  all)
    warn "logs all: use logs apicash | logs gatebox | logs banco em terminais separados"
    ac_logs_all
    ;;
  apicash) ac_logs_all ;;
  gatebox) gb_logs_all ;;
  banco) bb_logs_all ;;
  esac
}

case "${CMD}" in
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
