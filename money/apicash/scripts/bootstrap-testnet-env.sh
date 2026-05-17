#!/usr/bin/env bash
# Gera identidades testnet fundadas, emite BRLx (classic + SAC) e acrescenta variáveis a money/.env.
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APICASH="${MONEY}/apicash"
ENV_FILE="${MONEY}/.env"

export STELLAR_NETWORK_PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
export STELLAR_RPC_URL="${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}"
export APICASH_STELLAR_NETWORK_PASSPHRASE="${APICASH_STELLAR_NETWORK_PASSPHRASE:-$STELLAR_NETWORK_PASSPHRASE}"
export APICASH_SOROBAN_RPC_URL="${APICASH_SOROBAN_RPC_URL:-$STELLAR_RPC_URL}"

BIN="${APICASH_STELLAR_CLI_BIN:-stellar}"

for id in holdfy-deployer holdfy-buyer holdfy-seller; do
  if ! "$BIN" keys address "$id" >/dev/null 2>&1; then
    "$BIN" keys generate "$id" --fund --overwrite
  fi
done

DEPLOYER_G="$("$BIN" keys address holdfy-deployer)"
BUYER_G="$("$BIN" keys address holdfy-buyer)"
SELLER_G="$("$BIN" keys address holdfy-seller)"
DEPLOYER_S="$("$BIN" keys secret holdfy-deployer)"
BUYER_S="$("$BIN" keys secret holdfy-buyer)"

ASSET="BRLx:${DEPLOYER_G}"
if ! "$BIN" contract asset deploy --asset "$ASSET" --source-account holdfy-deployer 2>/dev/null | tee /tmp/brlx-deploy.out; then
  true
fi
TOKEN_ID="$(grep -E '^C[A-Z0-9]{55}$' /tmp/brlx-deploy.out 2>/dev/null | tail -1 || true)"
if [ -z "${TOKEN_ID}" ]; then
  TOKEN_ID="$("$BIN" contract id asset --asset "$ASSET" 2>/dev/null || true)"
fi

append_env() {
  local key="$1" val="$2"
  if grep -q "^${key}=" "$ENV_FILE" 2>/dev/null; then
    sed -i "s|^${key}=.*|${key}=${val}|" "$ENV_FILE"
  else
    printf '%s=%s\n' "$key" "$val" >>"$ENV_FILE"
  fi
}

touch "$ENV_FILE"
append_env APICASH_REQUIRE_TESTNET 1
append_env APICASH_SOROBAN_ENABLED 1
append_env APICASH_SOROBAN_STRICT 1
append_env APICASH_STELLAR_NETWORK testnet
append_env APICASH_SOROBAN_SOURCE_SECRET "$DEPLOYER_S"
append_env APICASH_SOROBAN_BUYER_SOURCE "$BUYER_S"
append_env APICASH_STELLAR_BUYER_ADDRESS "$BUYER_G"
append_env APICASH_STELLAR_SELLER_ADDRESS "$SELLER_G"
append_env APICASH_SOROBAN_ADMIN_ADDRESS "$DEPLOYER_G"
append_env APICASH_SOROBAN_PLATFORM_ADDRESS "$DEPLOYER_G"
append_env APICASH_SOROBAN_INIT 1
[ -n "${TOKEN_ID}" ] && append_env APICASH_BRLX_TOKEN_CONTRACT_ID "$TOKEN_ID"

ln -sfn "${ENV_FILE}" "${APICASH}/.env"

echo "[bootstrap] money/.env atualizado (testnet keys + token ${TOKEN_ID:-<run asset deploy>})"
echo "[bootstrap] deploy escrow: cd ${APICASH} && scripts/soroban-testnet-deploy.sh"
