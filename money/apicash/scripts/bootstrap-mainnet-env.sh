#!/usr/bin/env bash
# Gera identidades mainnet (SEM --fund), orienta compra de XLM real e acrescenta variáveis a money/.env.
#
# ATENÇÃO: Mainnet usa XLM real. As chaves geradas aqui NUNCA devem ser commitadas.
# Custo mínimo: ~2 XLM por conta (reserve base) + ~0.01 XLM por operação.
# Recomendado: fundir ao menos 10 XLM em cada conta antes de operar.
#
# Uso:
#   cd money/apicash
#   scripts/bootstrap-mainnet-env.sh
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APICASH="${MONEY}/apicash"
ENV_FILE="${MONEY}/.env"

BIN="${APICASH_STELLAR_CLI_BIN:-stellar}"

echo ""
echo "=== Bootstrap APICash — Stellar MAINNET ==="
echo "Rede: mainnet (XLM real)"
echo ""

# Verificar que stellar CLI está disponível
if ! command -v "$BIN" &>/dev/null; then
  echo "[ERRO] stellar CLI não encontrado. Instale: https://stellar.org/developers/stellar-cli"
  exit 1
fi

# Gerar (ou reutilizar) identidades mainnet
for id in holdfy-main-deployer holdfy-main-buyer holdfy-main-seller; do
  if "$BIN" keys address "$id" >/dev/null 2>&1; then
    echo "[skip] $id já existe"
  else
    "$BIN" keys generate "$id" --overwrite
    echo "[ok]   $id gerado"
  fi
done

DEPLOYER_G="$("$BIN" keys address holdfy-main-deployer)"
BUYER_G="$("$BIN" keys address holdfy-main-buyer)"
SELLER_G="$("$BIN" keys address holdfy-main-seller)"
DEPLOYER_S="$("$BIN" keys secret holdfy-main-deployer)"
BUYER_S="$("$BIN" keys secret holdfy-main-buyer)"

echo ""
echo "=== Endereços gerados ==="
echo "  Deployer/Issuer: $DEPLOYER_G"
echo "  Buyer:           $BUYER_G"
echo "  Seller:          $SELLER_G"
echo ""
echo "=== AÇÃO MANUAL NECESSÁRIA: Comprar XLM ==="
echo "  Antes de continuar (deploy do contrato), deposite XLM REAL em cada conta:"
echo ""
echo "  1. Compre XLM em exchange (Binance, Coinbase, Mercado Bitcoin, etc.)"
echo "  2. Envie pelo menos 10 XLM para cada endereço acima"
echo "  3. Verifique no explorer: https://stellar.expert/explorer/public/account/$DEPLOYER_G"
echo ""
echo "  Após fundas as contas, execute:"
echo "    cd ${APICASH} && scripts/soroban-mainnet-deploy.sh"
echo ""

append_env() {
  local key="$1" val="$2"
  if grep -q "^${key}=" "$ENV_FILE" 2>/dev/null; then
    sed -i "s|^${key}=.*|${key}=${val}|" "$ENV_FILE"
  else
    printf '%s=%s\n' "$key" "$val" >>"$ENV_FILE"
  fi
}

touch "$ENV_FILE"
append_env APICASH_REQUIRE_TESTNET 0
append_env APICASH_SOROBAN_ENABLED 1
append_env APICASH_SOROBAN_STRICT 1
append_env APICASH_STELLAR_NETWORK mainnet
append_env APICASH_STELLAR_HORIZON_URL "https://horizon.stellar.org"
append_env APICASH_SOROBAN_RPC_URL "https://rpc.stellar.org"
append_env APICASH_STELLAR_NETWORK_PASSPHRASE "Public Global Stellar Network ; September 2015"
append_env APICASH_SOROBAN_SOURCE_SECRET "$DEPLOYER_S"
append_env APICASH_SOROBAN_BUYER_SOURCE "$BUYER_S"
append_env APICASH_STELLAR_BUYER_ADDRESS "$BUYER_G"
append_env APICASH_STELLAR_SELLER_ADDRESS "$SELLER_G"
append_env APICASH_SOROBAN_ADMIN_ADDRESS "$DEPLOYER_G"
append_env APICASH_SOROBAN_PLATFORM_ADDRESS "$DEPLOYER_G"
# Issuer BRLx = mesmo deployer (conta que emite o token)
append_env APICASH_STELLAR_ISSUER_SECRET "$DEPLOYER_S"
append_env APICASH_STELLAR_ISSUER_ADDRESS "$DEPLOYER_G"
append_env APICASH_SOROBAN_INIT 1

ln -sfn "${ENV_FILE}" "${APICASH}/.env"

echo "[bootstrap] money/.env atualizado com identidades mainnet."
echo "[bootstrap] APICASH_SOROBAN_ESCROW_CONTRACT_ID e APICASH_BRLX_TOKEN_CONTRACT_ID"
echo "[bootstrap] serão preenchidos após scripts/soroban-mainnet-deploy.sh."
