#!/usr/bin/env bash
# Executa core, admin e whatsapp em paralelo (fallback quando npx/concurrently não está disponível).
set -euo pipefail
cd "$(dirname "$0")/.."

cleanup() {
  local p
  p=$(jobs -p)
  if [[ -n "$p" ]]; then
    kill $p 2>/dev/null || true
  fi
}
trap cleanup INT TERM EXIT

[[ -f ../.env ]] && set -a && source ../.env && set +a || true

export APICASH_HTTP_PORT="${APICASH_HTTP_PORT:-${API_PORT:-3000}}"

cargo run -p apicash-core &
cargo run -p apicash-admin-backend &
cargo run -p apicash-whatsapp &
wait
