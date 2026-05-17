#!/usr/bin/env bash
# Valida money/.env para APICASH_X402_REQUIRED=1 (HTTP 402 / Base Sepolia USDC).
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ENV_FILE="${MONEY}/.env"

set -a
[ -f "${ENV_FILE}" ] && . "${ENV_FILE}"
set +a

truthy() {
  case "${1:-}" in
  1 | true | TRUE | yes | YES | on | ON) return 0 ;;
  *) return 1 ;;
  esac
}

if ! truthy "${APICASH_X402_REQUIRED:-}"; then
  echo "[x402] APICASH_X402_REQUIRED não activo — validação omitida"
  exit 0
fi

missing=0
need() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    echo "[x402] missing: $name"
    missing=1
  fi
}

need X402_FACILITATOR_URL
need X402_PAY_TO

network="${X402_NETWORK:-base-sepolia}"
if [ "$network" != "base-sepolia" ] && [ "$network" != "base_sepolia" ]; then
  echo "[x402] X402_NETWORK deve ser base-sepolia (got '$network')"
  missing=1
fi

if [ "$missing" -ne 0 ]; then
  echo "[x402] configure x402 em money/.env (ver .env.example e README)"
  exit 1
fi

echo "[x402] OK — facilitator=${X402_FACILITATOR_URL} pay_to=${X402_PAY_TO} price=${X402_PRICE_USDC:-0.01} USDC"
