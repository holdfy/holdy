#!/usr/bin/env bash
# Infra Docker vive apenas em ../../ (money/runinfra.sh).
# Este script só sobe crates APICash no host depois da infra estar no ar.

set -euo pipefail
APICASH="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MONEY="$(cd "${APICASH}/.." && pwd)"
cd "${APICASH}"

export PATH="${PATH:-}"

echo "[1/4] Infra (money/runinfra.sh)…"
"${MONEY}/runinfra.sh" start

unset APICASH_AUTH_USERS || true

echo "[2/4] apicash-core :3000…"
APICASH_HTTP_PORT=3000 cargo run -p apicash-core &
PID_CORE=$!

echo "[3/4] apicash-admin-backend :3001…"
ADMIN_PORT=3001 cargo run -p apicash-admin-backend &
PID_ADMIN=$!

echo "[4/4] apicash-frontend (Leptos) :3002…"
cargo run -p apicash-frontend --features ssr &
PID_UI=$!

echo "PIDs: core=$PID_CORE admin=$PID_ADMIN ui=$PID_UI"
echo "Abrir no browser:"
echo "  http://127.0.0.1:3002  — dashboard (UI)"
echo "  http://127.0.0.1:3000  — API (GET / redireciona para /health)"
echo "  http://127.0.0.1:3001/health — admin API (sem chave)"
echo "Parar: kill $PID_CORE $PID_ADMIN $PID_UI ; ${MONEY}/runinfra.sh stop"

wait
