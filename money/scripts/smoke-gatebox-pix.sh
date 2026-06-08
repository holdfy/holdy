#!/usr/bin/env bash
# Smoke: Gatebox PIX IN (QR dinâmico) local.
# Pré-requisitos: infra (postgres gatebox); ./runapp.sh start gatebox
#               Gatebox com gateways configurados no .env (ex. PIX_GATEWAY_SULCRED + SULCRED_*).
#
# GATEBOX não vazio esperado em `.qrCode` do JSON quando o Sulcred está acessível
# (`SulcredHttpService` POST `{SULCRED_OUT_URL}/cob`; o mock expõe `/cob`).
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT="${ROOT:-}"

if [[ -z "${ROOT}" ]] && [[ -f "${MONEY}/.env" ]]; then
  # shellcheck source=/dev/null
  set -a && . "${MONEY}/.env" && set +a
fi

GB_PORT="${GB_API_PORT:-8081}"
H="${MONEY_LAN_HOST:-10.20.3.75}"
BASE="${GATEBOX_BASE_URL:-http://${H}:${GB_PORT}}"

body_json() {
  cat <<JSON
{"amount": 10.5,
 "payer_name": "Fulano Teste",
 "payer_document": "12345678909",
 "description": "Smoke pedido scripts/smoke-gatebox-pix",
 "expiration_seconds": 1800,
 "reference": "smoke-$(date +%s)",
 "pix_key": "teste@simulator.com"}
JSON
}

echo "→ POST ${BASE}/api/v1/pix/qrcode"
resp="$(curl -sS "${BASE}/api/v1/pix/qrcode" \
  -H 'Content-Type: application/json' \
  -d "$(body_json)")"
printf '%s\n' "${resp}"

if command -v jq >/dev/null 2>&1; then
  q="$(echo "${resp}" | jq -r '.qrCode // empty')"
  [[ -n "${q}" ]] || {
    echo "[smoke-gatebox-pix][ERRO] qrCode vazio — confirme configuração dos gateways no Gatebox (ex. SULCRED_OUT_URL), MESSAGING_BACKEND=sync."
    exit 1
  }
  echo "[smoke-gatebox-pix] qrCode comprimento=$(printf '%s' "${q}" | wc -c)"
else
  echo "(instale jq para validar qrCode automaticamente)"
fi
