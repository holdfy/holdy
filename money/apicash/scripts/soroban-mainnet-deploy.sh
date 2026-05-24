#!/usr/bin/env bash
# Deploy do BRLx SAC + contrato escrow na Stellar MAINNET.
#
# Pré-requisitos:
#   1. scripts/bootstrap-mainnet-env.sh já executado
#   2. Conta holdfy-main-deployer financiada com >= 10 XLM REAL
#   3. WASM compilado: cargo build --target wasm32v1-none --release -p apicash-soroban-contracts
#
# Uso:
#   cd money/apicash
#   scripts/soroban-mainnet-deploy.sh
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

set -a
[ -f .env ] && . ./.env
set +a

NETWORK="${APICASH_STELLAR_NETWORK:-mainnet}"

if [ "$NETWORK" != "mainnet" ]; then
  echo "[ERRO] Este script é exclusivo para mainnet."
  echo "       Use scripts/soroban-testnet-deploy.sh para testnet."
  exit 1
fi

export APICASH_SOROBAN_RPC_URL="${APICASH_SOROBAN_RPC_URL:-https://rpc.stellar.org}"
export APICASH_STELLAR_NETWORK_PASSPHRASE="${APICASH_STELLAR_NETWORK_PASSPHRASE:-Public Global Stellar Network ; September 2015}"
export APICASH_SOROBAN_WASM_PATH="${APICASH_SOROBAN_WASM_PATH:-target/wasm32v1-none/release/apicash_soroban_contracts.wasm}"

BIN="${APICASH_STELLAR_CLI_BIN:-stellar}"
MONEY="$(cd .. && pwd)"
ENV_FILE="${MONEY}/.env"

: "${APICASH_SOROBAN_SOURCE_SECRET:?Execute bootstrap-mainnet-env.sh primeiro}"
: "${APICASH_STELLAR_ISSUER_ADDRESS:?Execute bootstrap-mainnet-env.sh primeiro}"

echo ""
echo "=== Deploy APICash — Stellar MAINNET ==="
echo "RPC:  $APICASH_SOROBAN_RPC_URL"
echo "Rede: $APICASH_STELLAR_NETWORK_PASSPHRASE"
echo ""

# Verificar saldo do deployer antes de continuar
DEPLOYER_G="$("$BIN" keys address holdfy-main-deployer 2>/dev/null || echo '')"
if [ -z "$DEPLOYER_G" ]; then
  echo "[ERRO] Identidade holdfy-main-deployer não encontrada. Execute bootstrap-mainnet-env.sh."
  exit 1
fi

echo "[check] Deployer: $DEPLOYER_G"
echo "[check] Verificando saldo (requer conexão mainnet)..."
if ! "$BIN" account balance "$DEPLOYER_G" --network mainnet 2>/dev/null | grep -q "XLM"; then
  echo ""
  echo "[AVISO] Não foi possível verificar saldo do deployer."
  echo "        Certifique-se de que a conta está financiada com XLM antes de continuar."
  echo "        Explorer: https://stellar.expert/explorer/public/account/$DEPLOYER_G"
  echo ""
  read -rp "Continuar mesmo assim? (s/N): " confirm
  [[ "${confirm,,}" == "s" ]] || exit 0
fi

# Verificar WASM
if [ ! -f "${APICASH_SOROBAN_WASM_PATH}" ]; then
  echo "[build] WASM não encontrado — compilando..."
  cargo build --target wasm32v1-none --release -p apicash-soroban-contracts
fi

append_env() {
  local key="$1" val="$2"
  if grep -q "^${key}=" "$ENV_FILE" 2>/dev/null; then
    sed -i "s|^${key}=.*|${key}=${val}|" "$ENV_FILE"
  else
    printf '%s=%s\n' "$key" "$val" >>"$ENV_FILE"
  fi
}

# Passo 1: Deploy do BRLx SAC (Stellar Asset Contract)
echo ""
echo "[1/2] Deploy BRLx SAC na mainnet..."
ASSET="BRLx:${APICASH_STELLAR_ISSUER_ADDRESS}"
if ! "$BIN" contract asset deploy \
  --asset "$ASSET" \
  --source-account holdfy-main-deployer \
  --network mainnet \
  2>&1 | tee /tmp/brlx-mainnet-deploy.out; then
  echo "[aviso] Deploy BRLx SAC falhou ou já existe — tentando ler contract ID existente..."
fi

TOKEN_ID="$(grep -E '^C[A-Z0-9]{55}$' /tmp/brlx-mainnet-deploy.out 2>/dev/null | tail -1 || true)"
if [ -z "${TOKEN_ID}" ]; then
  TOKEN_ID="$("$BIN" contract id asset --asset "$ASSET" --network mainnet 2>/dev/null || true)"
fi

if [ -z "${TOKEN_ID}" ]; then
  echo "[ERRO] Não foi possível obter o ID do contrato BRLx."
  exit 1
fi

echo "[ok] BRLx SAC: $TOKEN_ID"
append_env APICASH_BRLX_TOKEN_CONTRACT_ID "$TOKEN_ID"

# Passo 2: Deploy do contrato escrow
echo ""
echo "[2/2] Deploy do contrato escrow na mainnet..."
cargo run -p apicash-cli -- deploy-contracts

# Nota: deploy-contracts deve gravar APICASH_SOROBAN_ESCROW_CONTRACT_ID em .env via append_env interno.
# Se não gravar automaticamente, o ID aparece no stdout — copie manualmente para .env.

echo ""
echo "=== Deploy mainnet concluído ==="
echo "  BRLx SAC:   $TOKEN_ID"
echo "  Explorer:   https://stellar.expert/explorer/public/contract/$TOKEN_ID"
echo ""
echo "Próximos passos:"
echo "  1. Confirme APICASH_SOROBAN_ESCROW_CONTRACT_ID em money/.env"
echo "  2. Inicie a stack: cd money && ./runapp.sh"
echo "  3. Smoke test:    cd money/apicash && scripts/testnet-onchain-smoke.sh  # ajuste APICASH_STELLAR_NETWORK=mainnet"
