#!/usr/bin/env bash
# Smoke NFS-e — mesmo fluxo documentado em docs/ideia.txt e PLANO_EXECUCAO.md
# Uso: NFSE_INSCRICAO=... NFSE_SENHA=... ./scripts/nfse-cpf-lookup.sh 86481096987
set -euo pipefail

CPF="${1:-86481096987}"
CPF="${CPF//[^0-9]/}"

INSCRICAO="${NFSE_INSCRICAO:-28.805.791/0001-46}"
SENHA="${NFSE_SENHA:-Senha1234*}"

COOKIE_FILE="$(mktemp)"
trap 'rm -f "$COOKIE_FILE"' EXIT

LOGIN_URL="https://www.nfse.gov.br/EmissorNacional/Login"
HTML=$(curl -fsS -c "$COOKIE_FILE" "$LOGIN_URL")
TOKEN=$(echo "$HTML" | sed -n 's/.*name="__RequestVerificationToken"[^>]*value="\([^"]*\)".*/\1/p' | head -n 1)

echo ">> Login (Inscricao + Senha)..."
curl -fsS -b "$COOKIE_FILE" -c "$COOKIE_FILE" \
  -X POST "$LOGIN_URL" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -H "Referer: $LOGIN_URL" \
  --data-urlencode "__RequestVerificationToken=$TOKEN" \
  --data-urlencode "Inscricao=$INSCRICAO" \
  --data-urlencode "Senha=$SENHA" \
  -o /dev/null -w "HTTP: %{http_code}\n"

echo ">> GET RecuperarInfoInscricao/$CPF"
curl -fsS -b "$COOKIE_FILE" \
  "https://www.nfse.gov.br/emissornacional/api/EmissaoDPS/RecuperarInfoInscricao/${CPF}?data=$(date +%Y-%m-%d)" \
  -H "Accept: application/json" \
  -H "Referer: https://www.nfse.gov.br/EmissorNacional/" \
  | python3 -m json.tool
