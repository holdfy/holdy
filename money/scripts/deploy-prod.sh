#!/usr/bin/env bash
# deploy-prod.sh — build completo + deploy no SaveInCloud (tudo num só comando)
# Uso: cd /home/holdfy/git/holdy/money && ./scripts/deploy-prod.sh
set -euo pipefail

SERVER="268389-38415@gate.paas.saveincloud.net.br"
SSH_PORT="3022"
PROD_HOST="holdfy-dev.sp1.br.saveincloud.net.br"
PROD_IP="${PROD_HOST}"
PROD_DASH_PORT="${PROD_DASH_PORT:-9091}"

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOLDY_ROOT="$(cd "${MONEY}/.." && pwd)"
APICASH="${MONEY}/apicash"
GATEBOX="${HOLDY_ROOT}/gatebox/gatebox-rust"
BANCO_BE="${HOLDY_ROOT}/gatebox/banco/backend_banco"
DEPLOY_DIR=/tmp/holdy-deploy

MUSL_LINKER="${HOME}/.local/musl/x86_64-linux-musl-cross/bin/x86_64-linux-musl-gcc"
MUSL_TARGET="x86_64-unknown-linux-musl"

ssh_run() { ssh -p "$SSH_PORT" -o ConnectTimeout=30 -o BatchMode=yes -o ServerAliveInterval=30 -o ServerAliveCountMax=10 "$SERVER" "$@"; }

_BROWSER=""
_find_browser() {
  [ -n "${_BROWSER}" ] && return 0
  local b
  for b in google-chrome google-chrome-stable chromium chromium-browser brave-browser microsoft-edge vivaldi; do
    if command -v "$b" >/dev/null 2>&1; then
      _BROWSER="$b"
      return 0
    fi
  done
  return 1
}

_open_url() {
  local label="$1" url="$2"
  echo "  browser → ${label}: ${url}"
  if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    return 0
  fi
  if _find_browser; then
    "${_BROWSER}" --new-tab "${url}" >/dev/null 2>&1 &
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "${url}" >/dev/null 2>&1 &
  elif command -v gio >/dev/null 2>&1; then
    gio open "${url}" >/dev/null 2>&1 &
  fi
}

_serve_prod_dash() {
  fuser "${PROD_DASH_PORT}/tcp" >/dev/null 2>&1 && return 0
  mkdir -p "${MONEY}/.runapp/prod-dash"
  python3 -m http.server "${PROD_DASH_PORT}" --directory "${HOLDY_ROOT}" \
    >"${MONEY}/.runapp/prod-dash/server.log" 2>&1 &
  disown
  sleep 0.4
}

echo "╔══════════════════════════════════════════════╗"
echo "║  HoldFy — Build + Deploy → SaveInCloud       ║"
echo "╚══════════════════════════════════════════════╝"
echo ""

# ── 1. Build APICash (Rust, musl) ─────────────────────────────────────────────
echo "[1/5] Build APICash (Rust → musl)..."
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="${MUSL_LINKER}"

(
  cd "${APICASH}"
  cargo build --release --target "${MUSL_TARGET}" \
    -p apicash-core \
    -p apicash-admin-backend \
    -p apicash-whatsapp \
    --features soroban
  cargo build --release --target "${MUSL_TARGET}" \
    -p apicash-frontend --no-default-features --features ssr
)

echo "  APICash OK"

# ── 2. Build Gatebox (Rust, musl) ─────────────────────────────────────────────
echo ""
echo "[2/5] Build Gatebox (Rust → musl)..."
(
  cd "${GATEBOX}"
  cargo build --release --target "${MUSL_TARGET}"
)
echo "  Gatebox OK"

# ── 3. Build backend_banco (Go, linux/amd64) ──────────────────────────────────
echo ""
echo "[3/5] Build backend_banco (Go → linux/amd64)..."
mkdir -p "${DEPLOY_DIR}"
(
  cd "${BANCO_BE}"
  GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -o "${DEPLOY_DIR}/backend_banco" ./cmd/server/
)
echo "  backend_banco OK"

# ── 4. Strip + empacotar ──────────────────────────────────────────────────────
echo ""
echo "[4/5] Strip + pacote..."

APICASH_T="${APICASH}/target/${MUSL_TARGET}/release"
GATEBOX_T="${GATEBOX}/target/${MUSL_TARGET}/release"

for bin in apicash-core apicash-admin-backend apicash-frontend apicash-whatsapp; do
  cp "${APICASH_T}/${bin}" "${DEPLOY_DIR}/${bin}"
  strip "${DEPLOY_DIR}/${bin}" 2>/dev/null || true
done

GATEBOX_BIN=$(find "${GATEBOX_T}" -maxdepth 1 -type f -executable ! -name "*.d" | head -1)
cp "${GATEBOX_BIN}" "${DEPLOY_DIR}/gatebox-rust"
strip "${DEPLOY_DIR}/gatebox-rust" 2>/dev/null || true
strip "${DEPLOY_DIR}/backend_banco" 2>/dev/null || true

echo "  Binários prontos:"
ls -lh "${DEPLOY_DIR}"/ | awk 'NR>1 {printf "  %-30s %s\n", $NF, $5}'

# ── 5. Upload → stop → install → start → browser ─────────────────────────────
echo ""
echo "[5/5] Deploy no servidor..."

echo "  Empacotando binários..."
TAR_PKG="${DEPLOY_DIR}/holdy-bins.tar.gz"
tar -czf "${TAR_PKG}" -C "${DEPLOY_DIR}" \
  apicash-core apicash-admin-backend apicash-frontend apicash-whatsapp gatebox-rust backend_banco
printf "  pacote: "; ls -lh "${TAR_PKG}" | awk '{print $5}'

echo "  Enviando pacote único (1 conexão SCP)..."
scp -P "${SSH_PORT}" -o ConnectTimeout=60 -o ServerAliveInterval=30 \
  "${TAR_PKG}" "${SERVER}:/tmp/holdy-bins.tar.gz"
echo "  enviado."

echo ""
echo "  Parando stack..."
ssh_run 'bash -s' <<'STOPREMOTE'
~/holdy/stop.sh 2>/dev/null || true
STOPREMOTE

echo "  Instalando binários e aplicando configuração de produção..."
ssh_run 'bash -s' <<INSTALLREMOTE
set -e
mkdir -p "\$HOME/holdy/bin" "\$HOME/holdy/config"
tar -xzf /tmp/holdy-bins.tar.gz -C "\$HOME/holdy/bin/"
chmod +x "\$HOME/holdy/bin"/apicash-core \\
         "\$HOME/holdy/bin"/apicash-admin-backend \\
         "\$HOME/holdy/bin"/apicash-frontend \\
         "\$HOME/holdy/bin"/apicash-whatsapp \\
         "\$HOME/holdy/bin"/gatebox-rust \\
         "\$HOME/holdy/bin"/backend_banco
rm -f /tmp/holdy-bins.tar.gz
echo "  Binários instalados."

# ── Patch .env para valores de produção (HTTPS + IPs internos) ───────────────
ENV_FILE="\$HOME/holdy/money/.env"
if [ -f "\$ENV_FILE" ]; then
  sed -i \\
    -e 's|PUBLIC_APP_URL=.*|PUBLIC_APP_URL=https://${PROD_HOST}|' \\
    -e 's|APICASH_HTTP_BASE=http://192[.0-9]*:[0-9]*|APICASH_HTTP_BASE=https://${PROD_HOST}/svc/core|' \\
    -e 's|APICASH_ADMIN_HTTP_BASE=http://192[.0-9]*:[0-9]*|APICASH_ADMIN_HTTP_BASE=https://${PROD_HOST}/svc/admin|' \\
    -e 's|APICASH_CORE_URL=http://192[.0-9]*:[0-9]*|APICASH_CORE_URL=http://127.0.0.1:3000|' \\
    -e 's|ADMIN_API_URL=http://192[.0-9]*:[0-9]*|ADMIN_API_URL=http://127.0.0.1:3001|' \\
    -e 's|APICASH_ADMIN_API_URL=http://192[.0-9]*:[0-9]*|APICASH_ADMIN_API_URL=http://127.0.0.1:3001|' \\
    -e 's|GATEBOX_BASE_URL=http://192[.0-9]*:[0-9]*|GATEBOX_BASE_URL=http://127.0.0.1:8081|' \\
    -e 's|BANCO_WEBHOOK_URL=http://192[.0-9]*:[0-9]*/|BANCO_WEBHOOK_URL=http://127.0.0.1:8081/|' \\
    -e 's|LOGISTICA_WHATSAPP_NOTIFY_URL=http://192[.0-9]*:[0-9]*/|LOGISTICA_WHATSAPP_NOTIFY_URL=http://127.0.0.1:3010/|' \\
    -e 's|SCRAPER_URL=http://192[.0-9]*:[0-9]*|SCRAPER_URL=http://127.0.0.1:4000|' \\
    -e 's|SULCRED_OUT_URL=http://192[.0-9]*:[0-9]*|SULCRED_OUT_URL=http://127.0.0.1:7020|' \\
    -e 's|APICASH_TRACKING_SIMULATOR_URL=http://192[.0-9]*:[0-9]*|APICASH_TRACKING_SIMULATOR_URL=http://127.0.0.1:8092|' \\
    -e 's|DATABASE_URL=postgresql://[^@]*@192[.0-9]*:[0-9]*/|DATABASE_URL=postgresql://apicash:apicash@10.100.85.142:5432/|' \\
    -e 's|BANCO_DATABASE_URL=postgresql://[^@]*@192[.0-9]*:[0-9]*/|BANCO_DATABASE_URL=postgresql://apicash:apicash@10.100.85.142:5432/|' \\
    -e 's|POSTGRESQL_WRITE_URL=postgres://[^@]*@192[.0-9]*:[0-9]*/|POSTGRESQL_WRITE_URL=postgres://apicash:apicash@10.100.85.142:5432/|' \\
    -e 's|POSTGRESQL_READ_URL=postgres://[^@]*@192[.0-9]*:[0-9]*/|POSTGRESQL_READ_URL=postgres://apicash:apicash@10.100.85.142:5432/|' \\
    -e 's|REDIS_URL=redis://192[.0-9]*:[0-9]*/|REDIS_URL=redis://127.0.0.1:6379/|' \\
    "\$ENV_FILE"
  echo "  .env produção atualizado (HTTPS + IPs internos)."
else
  echo "  AVISO: \$ENV_FILE não encontrado — crie antes de iniciar."
fi

~/holdy/run.sh
INSTALLREMOTE

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║  Deploy concluído!                           ║"
echo "╚══════════════════════════════════════════════╝"
echo "  Verificar: ssh -p ${SSH_PORT} ${SERVER} '~/holdy/status.sh'"
echo ""

# ── Buscar QR do WhatsApp do servidor de produção ────────────────────────────
_fetch_prod_qr() {
  local server_qr="/home/jelastic/.runapp/whatsapp-pairing-qr.png"
  local qr_dir="${HOLDY_ROOT}/whatsapp_qrcode"
  local local_qr="${qr_dir}/whatsapp-pairing-qr.png"
  local pair_src="${MONEY}/scripts/whatsapp-pair.html"
  local pair_dst="${qr_dir}/pair.html"

  mkdir -p "${qr_dir}"
  [ -f "${pair_src}" ] && cp -f "${pair_src}" "${pair_dst}"

  echo "  Aguardando QR WhatsApp no servidor (até 30s)..."
  local i=0
  while [[ $i -lt 30 ]]; do
    if ssh -p "${SSH_PORT}" -o ConnectTimeout=5 -o BatchMode=yes "${SERVER}" \
        "test -s ${server_qr}" 2>/dev/null; then
      scp -P "${SSH_PORT}" -q "${SERVER}:${server_qr}" "${local_qr}"
      echo "  QR copiado: ${local_qr}"
      return 0
    fi
    sleep 1
    i=$((i+1))
  done
  echo "  QR ainda não gerado (WhatsApp já pareado ou erro de start)"
  return 1
}

# ── Abrir browser ─────────────────────────────────────────────────────────────
_serve_prod_dash

if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
  echo "  (sem DISPLAY — abra manualmente)"
  echo "  prod-dashboard : http://127.0.0.1:${PROD_DASH_PORT}/prod-dashboard.html"
  echo "  QR WhatsApp    : http://127.0.0.1:${PROD_DASH_PORT}/whatsapp_qrcode/pair.html"
  echo "  HoldFy Site    : https://${PROD_HOST}/"
  echo "  APICash Core   : https://${PROD_HOST}/svc/core/health"
  echo "  APICash Admin  : https://${PROD_HOST}/svc/admin/health"
  echo "  WhatsApp health: https://${PROD_HOST}/svc/whatsapp/health"
  echo "  Gatebox PIX    : https://${PROD_HOST}/svc/gatebox/health"
  echo "  backend_banco  : https://${PROD_HOST}/svc/banco/health"
  _fetch_prod_qr || true
  exit 0
fi

_open_url "prod-dashboard"      "http://127.0.0.1:${PROD_DASH_PORT}/prod-dashboard.html"
sleep 0.5

# QR do WhatsApp — mesmo mecanismo do runapp.sh (pair.html local com PNG do servidor)
if _fetch_prod_qr; then
  sleep 0.4
  _open_url "QR WhatsApp"  "http://127.0.0.1:${PROD_DASH_PORT}/whatsapp_qrcode/pair.html"
fi
sleep 0.4

_open_url "HoldFy Site"         "https://${PROD_HOST}/"
sleep 0.4
_open_url "APICash Core"        "https://${PROD_HOST}/svc/core/health"
sleep 0.4
_open_url "APICash Admin"       "https://${PROD_HOST}/svc/admin/health"
sleep 0.4
_open_url "WhatsApp health"     "https://${PROD_HOST}/svc/whatsapp/health"
sleep 0.4
_open_url "Gatebox PIX"         "https://${PROD_HOST}/svc/gatebox/health"
sleep 0.4
_open_url "backend_banco"       "https://${PROD_HOST}/svc/banco/health"
echo ""
echo "  prod-dashboard: http://127.0.0.1:${PROD_DASH_PORT}/prod-dashboard.html"
