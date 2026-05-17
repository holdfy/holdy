#!/usr/bin/env bash
# Smoke: rota protegida sem JWT deve devolver HTTP 402 quando x402 está activo.
set -euo pipefail

APICASH_URL="${APICASH_URL:-http://127.0.0.1:3000}"
MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="${MONEY}/.env"

set -a
[ -f "${ENV_FILE}" ] && . "${ENV_FILE}"
set +a

if [ "${APICASH_X402_REQUIRED:-0}" != "1" ]; then
  echo "[x402-smoke] APICASH_X402_REQUIRED não é 1 — defina em money/.env e reinicie apicash-core"
  exit 1
fi

echo "[x402-smoke] GET ${APICASH_URL}/orders/00000000-0000-0000-0000-000000000001 (sem auth, sem pagamento)"
code=$(curl -sS -o /tmp/x402-smoke-body.txt -w '%{http_code}' \
  "${APICASH_URL}/orders/00000000-0000-0000-0000-000000000001")

echo "[x402-smoke] HTTP ${code}"
head -c 500 /tmp/x402-smoke-body.txt
echo

if [ "$code" != "402" ]; then
  echo "[x402-smoke] esperado 402 Payment Required"
  exit 1
fi

echo "[x402-smoke] OK — x402 activo (pagar com cliente x402 e repetir o pedido para obter 200/404)"
