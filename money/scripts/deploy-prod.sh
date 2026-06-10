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

SITE_DIR="${HOLDY_ROOT}/site"
HOLDFY_ADMIN_DIR="${HOLDY_ROOT}/holdfy-admin"
FRONT_GATEBOX_DIR="${HOLDY_ROOT}/gatebox/front-gatebox"
FRONT_HOLDY_DIR="${HOLDY_ROOT}/gatebox/front-holdy"

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
echo "[1/6] Build APICash (Rust → musl)..."
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="${MUSL_LINKER}"

(
  cd "${APICASH}"
  cargo build --release --target "${MUSL_TARGET}" \
    -p apicash-core \
    -p apicash-admin-backend \
    -p apicash-whatsapp
  cargo build --release --target "${MUSL_TARGET}" \
    -p apicash-frontend --no-default-features --features ssr
)

echo "  APICash OK"

# ── 1b. Build WASM/JS (Leptos hydration — gnu target, para wasm32) ────────────
echo ""
echo "[1b/6] Build Leptos WASM/JS..."
(
  cd "${APICASH}"
  cargo leptos build --release 2>&1 | tail -5
)
echo "  WASM OK"

# ── 2. Build Gatebox (Rust, musl) ─────────────────────────────────────────────
echo ""
echo "[2/6] Build Gatebox (Rust → musl)..."
(
  cd "${GATEBOX}"
  cargo build --release --target "${MUSL_TARGET}"
)
echo "  Gatebox OK"

# ── 3. Build backend_banco (Go, linux/amd64) ──────────────────────────────────
echo ""
echo "[3/6] Build backend_banco (Go → linux/amd64)..."
mkdir -p "${DEPLOY_DIR}"
(
  cd "${BANCO_BE}"
  GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -o "${DEPLOY_DIR}/backend_banco" ./cmd/server/
)
echo "  backend_banco OK"

# ── 3b. Build site React (holdy/site) ─────────────────────────────────────────
echo ""
echo "[3b/6] Build site HoldFy (React → /site/)..."
(
  cd "${SITE_DIR}"
  bun run build 2>&1 | tail -3
)
echo "  site OK"

# ── 3c. Build holdfy-admin (React) ────────────────────────────────────────────
echo ""
echo "[3c/6] Build holdfy-admin (React → /holdfy-admin/)..."
(
  cd "${HOLDFY_ADMIN_DIR}"
  bun run build 2>&1 | tail -3
)
echo "  holdfy-admin OK"

# ── 3d. Build front-gatebox (React) ───────────────────────────────────────────
echo ""
echo "[3d/6] Build front-gatebox (React → /front-gatebox/)..."
(
  cd "${FRONT_GATEBOX_DIR}"
  bun run build 2>&1 | tail -3
)
echo "  front-gatebox OK"

# ── 3e. Build front-holdy (React — vendedor/comprador) ────────────────────────
echo ""
echo "[3e/6] Build front-holdy (React → /front-holdy/)..."
(
  cd "${FRONT_HOLDY_DIR}"
  bun run build 2>&1 | tail -3
)
echo "  front-holdy OK"

# ── 4. Strip + empacotar ──────────────────────────────────────────────────────
echo ""
echo "[4/6] Strip + pacote..."

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
echo "[5/6] Deploy no servidor..."

echo "  Empacotando binários..."
TAR_PKG="${DEPLOY_DIR}/holdy-bins.tar.gz"
tar -czf "${TAR_PKG}" -C "${DEPLOY_DIR}" \
  apicash-core apicash-admin-backend apicash-frontend apicash-whatsapp gatebox-rust backend_banco
printf "  pacote: "; ls -lh "${TAR_PKG}" | awk '{print $5}'

echo "  Enviando binários..."
scp -P "${SSH_PORT}" -o ConnectTimeout=60 -o ServerAliveInterval=30 \
  "${TAR_PKG}" "${SERVER}:/tmp/holdy-bins.tar.gz"

echo "  Enviando assets WASM/JS (Leptos pkg/)..."
PKG_DIR="${APICASH}/target/site/pkg"
scp -P "${SSH_PORT}" -o ConnectTimeout=30 \
  "${PKG_DIR}/apicash_frontend.wasm" \
  "${PKG_DIR}/apicash_frontend.js" \
  "${SERVER}:/home/jelastic/pkg/"
echo "  assets enviados."

echo "  Enviando site React (holdy/site → /home/jelastic/site-dist/)..."
ssh_run "mkdir -p /home/jelastic/site-dist /home/jelastic/holdfy-admin-dist"
scp -P "${SSH_PORT}" -r -q "${SITE_DIR}/dist/." "${SERVER}:/home/jelastic/site-dist/"
echo "  site enviado."

echo "  Enviando holdfy-admin (React → /home/jelastic/holdfy-admin-dist/)..."
scp -P "${SSH_PORT}" -r -q "${HOLDFY_ADMIN_DIR}/dist/." "${SERVER}:/home/jelastic/holdfy-admin-dist/"
echo "  holdfy-admin enviado."

echo "  Enviando front-gatebox (React → /home/jelastic/front-gatebox-dist/)..."
ssh_run "mkdir -p /home/jelastic/front-gatebox-dist"
scp -P "${SSH_PORT}" -r -q "${FRONT_GATEBOX_DIR}/build/." "${SERVER}:/home/jelastic/front-gatebox-dist/"
echo "  front-gatebox enviado."

echo "  Enviando front-holdy (React → /home/jelastic/front-holdy-dist/)..."
ssh_run "mkdir -p /home/jelastic/front-holdy-dist"
scp -P "${SSH_PORT}" -r -q "${FRONT_HOLDY_DIR}/dist/." "${SERVER}:/home/jelastic/front-holdy-dist/"
echo "  front-holdy enviado."

echo ""
echo "  Parando stack..."
ssh_run 'bash -s' <<'STOPREMOTE'
~/holdy/stop.sh 2>/dev/null || true
STOPREMOTE

echo "  Instalando binários e reiniciando..."
ssh_run 'bash -s' <<'INSTALLREMOTE'
set -e
mkdir -p "$HOME/holdy/bin" "$HOME/holdy/config"
tar -xzf /tmp/holdy-bins.tar.gz -C "$HOME/holdy/bin/"
chmod +x "$HOME/holdy/bin"/apicash-core \
         "$HOME/holdy/bin"/apicash-admin-backend \
         "$HOME/holdy/bin"/apicash-frontend \
         "$HOME/holdy/bin"/apicash-whatsapp \
         "$HOME/holdy/bin"/gatebox-rust \
         "$HOME/holdy/bin"/backend_banco
rm -f /tmp/holdy-bins.tar.gz
echo "  Binários instalados."
echo ""
~/holdy/run.sh
INSTALLREMOTE

echo ""
echo "[6/6] Aguardando serviços ficarem prontos..."
_wait_ready() {
  local url="$1" label="$2" waited=0
  printf "  aguardando %-25s " "${label}…"
  while true; do
    code=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 3 "${url}" 2>/dev/null)
    if [[ "${code}" == "200" ]]; then
      echo "✓ (${waited}s)"
      return 0
    fi
    if [[ "${waited}" -ge 60 ]]; then
      echo "TIMEOUT (último: ${code})"
      return 1
    fi
    sleep 2; waited=$((waited+2)); printf "."
  done
}

_wait_ready "http://${PROD_HOST}/"                    "frontend"      || true
_wait_ready "http://${PROD_HOST}/"                    "site"          || true
_wait_ready "http://${PROD_HOST}/holdfy-admin/"       "holdfy-admin"  || true
_wait_ready "http://${PROD_HOST}/front-gatebox/"      "front-gatebox" || true
_wait_ready "http://${PROD_HOST}/front-holdy/"        "front-holdy"   || true
_wait_ready "http://${PROD_HOST}/svc/core/health"     "apicash-core"  || true
_wait_ready "http://${PROD_HOST}/svc/admin/health"    "apicash-admin" || true
_wait_ready "http://${PROD_HOST}/svc/gatebox/health"  "gatebox"       || true
_wait_ready "http://${PROD_HOST}/svc/banco/health"    "backend_banco" || true
_wait_ready "http://${PROD_HOST}/svc/whatsapp/health" "whatsapp"      || true

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

# ── Abrir browser (mesmas páginas que runapp.sh, mas apontando para o servidor) ──
_serve_prod_dash

if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
  echo "  (sem DISPLAY — abra manualmente)"
  echo "  prod-dashboard : http://127.0.0.1:${PROD_DASH_PORT}/prod-dashboard.html"
  echo "  QR WhatsApp    : http://127.0.0.1:${PROD_DASH_PORT}/whatsapp_qrcode/pair.html"
  echo "  Site HoldFy    : http://${PROD_HOST}/"
  echo "  holdfy-admin   : http://${PROD_HOST}/holdfy-admin/"
  echo "  front-gatebox  : http://${PROD_HOST}/front-gatebox/"
  echo "  front-holdy    : http://${PROD_HOST}/front-holdy/"
  _fetch_prod_qr || true
  exit 0
fi

# 1. prod-dashboard (equivalente ao dev-dashboard.html do runapp.sh)
_open_url "prod-dashboard"    "http://127.0.0.1:${PROD_DASH_PORT}/prod-dashboard.html"
sleep 0.5

# 2. QR WhatsApp (igual ao runapp.sh)
if _fetch_prod_qr; then
  sleep 0.4
  _open_url "QR WhatsApp"     "http://127.0.0.1:${PROD_DASH_PORT}/whatsapp_qrcode/pair.html"
fi
sleep 0.4

# 3. Site HoldFy (equivalente ao http://127.0.0.1:5173/ do runapp.sh)
_open_url "Site HoldFy"       "http://${PROD_HOST}/"
sleep 0.4

# 4. holdfy-admin (equivalente ao http://127.0.0.1:3020/ do runapp.sh)
_open_url "holdfy-admin"      "http://${PROD_HOST}/holdfy-admin/"
sleep 0.4

# 5. front-gatebox (equivalente ao http://127.0.0.1:3030/#/ do runapp.sh)
_open_url "front-gatebox"     "http://${PROD_HOST}/front-gatebox/"
sleep 0.4

# 6. front-holdy (vendedor/comprador)
_open_url "front-holdy"       "http://${PROD_HOST}/front-holdy/"

echo ""
echo "  prod-dashboard : http://127.0.0.1:${PROD_DASH_PORT}/prod-dashboard.html"
echo "  Site HoldFy    : http://${PROD_HOST}/"
echo "  holdfy-admin   : http://${PROD_HOST}/holdfy-admin/"
echo "  front-gatebox  : http://${PROD_HOST}/front-gatebox/"
echo "  front-holdy    : http://${PROD_HOST}/front-holdy/"
