#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

set -a
[ -f .env ] && . ./.env
set +a

missing=0
need() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    echo "[soroban] missing: $name"
    missing=1
  fi
}

if ! command -v "${APICASH_STELLAR_CLI_BIN:-stellar}" >/dev/null 2>&1; then
  echo "[soroban] missing: stellar CLI (cargo install stellar-cli --version 23.0.1 --locked --no-default-features)"
  missing=1
fi

export APICASH_SOROBAN_RPC_URL="${APICASH_SOROBAN_RPC_URL:-${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}}"
export APICASH_STELLAR_NETWORK_PASSPHRASE="${APICASH_STELLAR_NETWORK_PASSPHRASE:-${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}}"
export APICASH_SOROBAN_WASM_PATH="${APICASH_SOROBAN_WASM_PATH:-target/wasm32v1-none/release/apicash_soroban_contracts.wasm}"

need APICASH_SOROBAN_SOURCE_SECRET
need APICASH_SOROBAN_BUYER_SOURCE
need APICASH_STELLAR_BUYER_ADDRESS
need APICASH_STELLAR_SELLER_ADDRESS
need APICASH_BRLX_TOKEN_CONTRACT_ID

echo "[soroban] building wasm..."
rustup target add wasm32v1-none >/dev/null
(cd soroban-contracts && stellar contract build)

if [ ! -f "$APICASH_SOROBAN_WASM_PATH" ]; then
  echo "[soroban] wasm not found at $APICASH_SOROBAN_WASM_PATH"
  missing=1
fi

if [ "$missing" != 0 ]; then
  cat <<'MSG'

[soroban] Testnet não está pronto. Defina pelo menos:
  APICASH_SOROBAN_SOURCE_SECRET=<S... testnet funded>
  APICASH_SOROBAN_BUYER_SOURCE=<S... ou identidade local do buyer>
  APICASH_STELLAR_BUYER_ADDRESS=<G...>
  APICASH_STELLAR_SELLER_ADDRESS=<G...>
  APICASH_BRLX_TOKEN_CONTRACT_ID=<C... SEP-41 token testnet>

Depois rode:
  scripts/soroban-testnet-deploy.sh

MSG
  exit 1
fi

echo "[soroban] ready"
