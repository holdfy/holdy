#!/usr/bin/env bash
# Valida money/.env para APICASH_REQUIRE_TESTNET=1 (transações visíveis na Stellar testnet).
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

if ! truthy "${APICASH_REQUIRE_TESTNET:-}"; then
  echo "[testnet] APICASH_REQUIRE_TESTNET não activo — validação omitida"
  exit 0
fi

export APICASH_SOROBAN_ENABLED="${APICASH_SOROBAN_ENABLED:-1}"
export APICASH_SOROBAN_STRICT="${APICASH_SOROBAN_STRICT:-1}"
export APICASH_STELLAR_NETWORK="${APICASH_STELLAR_NETWORK:-${STELLAR_NETWORK:-testnet}}"

missing=0
need() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    echo "[testnet] missing: $name"
    missing=1
  fi
}

if [ "${APICASH_STELLAR_NETWORK}" != "testnet" ] && [ "${STELLAR_NETWORK:-testnet}" != "testnet" ]; then
  echo "[testnet] APICASH_STELLAR_NETWORK deve ser testnet"
  missing=1
fi

need APICASH_SOROBAN_ESCROW_CONTRACT_ID
need APICASH_BRLX_TOKEN_CONTRACT_ID
need APICASH_STELLAR_BUYER_ADDRESS
need APICASH_STELLAR_SELLER_ADDRESS

if [[ "${APICASH_SOROBAN_ESCROW_CONTRACT_ID:-}" == *mock* ]]; then
  echo "[testnet] APICASH_SOROBAN_ESCROW_CONTRACT_ID não pode ser placeholder mock"
  missing=1
fi

if [ -z "${APICASH_SOROBAN_SOURCE_SECRET:-}" ] && [ -z "${APICASH_SOROBAN_BUYER_SOURCE:-}" ]; then
  echo "[testnet] missing: APICASH_SOROBAN_SOURCE_SECRET ou APICASH_SOROBAN_BUYER_SOURCE"
  missing=1
fi

rpc="${APICASH_SOROBAN_RPC_URL:-${STELLAR_RPC_URL:-}}"
if [[ "${rpc}" != *testnet* ]] && [[ "${rpc}" != *futurenet* ]]; then
  echo "[testnet] RPC Soroban deve apontar para testnet (got: ${rpc:-<empty>})"
  missing=1
fi

if ! command -v "${APICASH_STELLAR_CLI_BIN:-stellar}" >/dev/null 2>&1; then
  echo "[testnet] missing: stellar CLI (stellar contract invoke/deploy)"
  missing=1
fi

if [ "$missing" != 0 ]; then
  cat <<MSG
[testnet] Configuração incompleta. Passos:

  1. cd money/apicash && scripts/soroban-testnet-check.sh
  2. scripts/soroban-testnet-deploy.sh   # preenche escrow + token no .env
  3. Em money/.env: APICASH_REQUIRE_TESTNET=1, APICASH_SOROBAN_ENABLED=1, APICASH_SOROBAN_STRICT=1
  4. ./runapp.sh build apicash && ./runapp.sh start apicash

Explorador: https://stellar.expert/explorer/testnet
MSG
  exit 1
fi

echo "[testnet] OK — on-chain obrigatório na Stellar testnet"
