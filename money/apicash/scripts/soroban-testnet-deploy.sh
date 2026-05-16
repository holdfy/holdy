#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

set -a
[ -f .env ] && . ./.env
set +a

export APICASH_SOROBAN_RPC_URL="${APICASH_SOROBAN_RPC_URL:-${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}}"
export APICASH_STELLAR_NETWORK_PASSPHRASE="${APICASH_STELLAR_NETWORK_PASSPHRASE:-${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}}"
export APICASH_SOROBAN_WASM_PATH="${APICASH_SOROBAN_WASM_PATH:-target/wasm32v1-none/release/apicash_soroban_contracts.wasm}"

if [ -z "${APICASH_SOROBAN_SOURCE_SECRET:-}" ] && [ -n "${APICASH_STELLAR_SECRET_KEY:-}" ]; then
  export APICASH_SOROBAN_SOURCE_SECRET="$APICASH_STELLAR_SECRET_KEY"
fi

if [ -z "${APICASH_SOROBAN_ADMIN_ADDRESS:-}" ] && [ -n "${APICASH_STELLAR_ADMIN_ADDRESS:-}" ]; then
  export APICASH_SOROBAN_ADMIN_ADDRESS="$APICASH_STELLAR_ADMIN_ADDRESS"
fi

if [ -z "${APICASH_SOROBAN_PLATFORM_ADDRESS:-}" ] && [ -n "${APICASH_STELLAR_PLATFORM_ADDRESS:-}" ]; then
  export APICASH_SOROBAN_PLATFORM_ADDRESS="$APICASH_STELLAR_PLATFORM_ADDRESS"
fi

scripts/soroban-testnet-check.sh

: "${APICASH_SOROBAN_SOURCE_SECRET:?missing APICASH_SOROBAN_SOURCE_SECRET}"

if [ -n "${APICASH_SOROBAN_ADMIN_ADDRESS:-}" ] && [ -n "${APICASH_SOROBAN_PLATFORM_ADDRESS:-}" ]; then
  export APICASH_SOROBAN_INIT="${APICASH_SOROBAN_INIT:-1}"
else
  echo "[soroban] APICASH_SOROBAN_ADMIN_ADDRESS/APICASH_SOROBAN_PLATFORM_ADDRESS ausentes; deploy sem initialize."
fi

echo "[soroban] deploying escrow contract..."
cargo run -p apicash-cli -- deploy-contracts
