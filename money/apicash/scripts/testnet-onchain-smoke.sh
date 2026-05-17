#!/usr/bin/env bash
# Smoke test: transfer BRLx → escrow + lock (tx reais na testnet). Imprime hashes para o explorador.
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APICASH="${MONEY}/apicash"
set -a
[ -f "${MONEY}/.env" ] && . "${MONEY}/.env"
set +a

export STELLAR_NETWORK_PASSPHRASE="${APICASH_STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
export STELLAR_RPC_URL="${APICASH_SOROBAN_RPC_URL:-https://soroban-testnet.stellar.org}"
BIN="${APICASH_STELLAR_CLI_BIN:-stellar}"

TOKEN="${APICASH_BRLX_TOKEN_CONTRACT_ID:?APICASH_BRLX_TOKEN_CONTRACT_ID}"
ESCROW="${APICASH_SOROBAN_ESCROW_CONTRACT_ID:?APICASH_SOROBAN_ESCROW_CONTRACT_ID}"
BUYER_G="${APICASH_STELLAR_BUYER_ADDRESS:?APICASH_STELLAR_BUYER_ADDRESS}"
AMOUNT="${1:-10000000}" # 1.0 BRLx com 7 decimais
ORDER_KEY="${2:-42}"

echo "[smoke] transfer BRLx → escrow..."
TRANSFER_OUT=$("$BIN" contract invoke --id "$TOKEN" --source holdfy-buyer --send=yes \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" --rpc-url "$STELLAR_RPC_URL" \
  -- transfer --from "$BUYER_G" --to "$ESCROW" --amount "$AMOUNT" 2>&1) || true
echo "$TRANSFER_OUT"

echo "[smoke] lock no contrato escrow (order_id=$ORDER_KEY)..."
LOCK_OUT=$("$BIN" contract invoke --id "$ESCROW" --source holdfy-buyer --send=yes \
  --network-passphrase "$STELLAR_NETWORK_PASSPHRASE" --rpc-url "$STELLAR_RPC_URL" \
  -- lock --order_id "$ORDER_KEY" --buyer "$BUYER_G" \
  --seller "${APICASH_STELLAR_SELLER_ADDRESS}" --token "$TOKEN" --amount "$AMOUNT" 2>&1) || true
echo "$LOCK_OUT"

EXPLORER="${APICASH_STELLAR_EXPLORER_BASE:-https://stellar.expert/explorer/testnet}"
echo ""
echo "[smoke] Ver transacções da conta comprador no Horizon/API:"
echo "  curl -s http://127.0.0.1:3000/testnet/transactions?limit=10"
echo "  curl -s \"${APICASH_STELLAR_HORIZON_URL:-https://horizon-testnet.stellar.org}/accounts/${BUYER_G}/transactions?order=desc&limit=10\""
echo "  Explorador: ${EXPLORER}/account/${BUYER_G}"
