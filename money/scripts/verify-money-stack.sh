#!/usr/bin/env bash
# Verificação rápida: infra + Gatebox + APICash (integração PIX WhatsApp → core → Gatebox).
set -euo pipefail

MONEY="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=/dev/null
[[ -f "${MONEY}/.env" ]] && set -a && . "${MONEY}/.env" && set +a

H="${MONEY_LAN_HOST:-10.20.3.75}"
CORE_PORT="${APICASH_HTTP_PORT:-${API_PORT:-3000}}"
GB_PORT="${GB_API_PORT:-8081}"
GB_URL="${GATEBOX_BASE_URL:-http://${H}:${GB_PORT}}"

ok() { printf '[verify] OK %s\n' "$*"; }
fail() { printf '[verify] FALHA %s\n' "$*" >&2; FAILED=1; }

FAILED=0

command -v curl >/dev/null 2>&1 || {
  echo "[verify] instale curl"
  exit 1
}

curl_code() {
  curl -sS -o /dev/null -w '%{http_code}' --max-time 3 "$1" 2>/dev/null || printf '000'
}

if docker info >/dev/null 2>&1; then
  if docker compose --project-directory "${MONEY}" -f "${MONEY}/docker-compose.yml" ps --status running 2>/dev/null | grep -q .; then
    ok "Docker compose (há containers em execução)"
  else
    fail "Docker compose sem containers — execute: ${MONEY}/runinfra.sh start"
  fi
else
  fail "Docker não acessível (opcional para só binários no host)"
fi

c="$(curl_code "http://${H}:${CORE_PORT}/health")"
if [[ "${c}" =~ ^2 ]]; then ok "apicash-core health (:${CORE_PORT})"; else fail "apicash-core não responde :${CORE_PORT} (./runapp.sh start apicash)"; fi

g="${GB_URL%/}"
c="$(curl_code "${g}/health")"
if [[ "${c}" =~ ^2 ]]; then ok "Gatebox Rust health (${g})"; else fail "Gatebox não responde ${g}/health (./runapp.sh start gatebox)"; fi

if command -v jq >/dev/null 2>&1; then
  body="$(curl -sS --max-time 15 "${g}/api/v1/pix/qrcode" \
    -H 'Content-Type: application/json' \
    -d '{"amount":1,"payer_name":"V","payer_document":"12345678909","description":"verify","expiration_seconds":600,"reference":"verify-1","pix_key":"test@simulator.com"}' 2>/dev/null || true)"
  q="$(printf '%s' "${body}" | jq -r '.qrCode // empty' 2>/dev/null || true)"
  if [[ -n "${q}" ]] && [[ "${q}" != *GATEBOXRUST* ]]; then
    ok "POST /api/v1/pix/qrcode devolve qrCode (EMV)"
  elif [[ "${q}" == *GATEBOXRUST* ]]; then
    fail "Gatebox devolve placeholder GATEBOXRUST — no .env use MESSAGING_BACKEND=sync (não pulsar+PULSAR_URL no processo Gatebox)"
  else
    fail "POST /pix/qrcode sem qrCode — no Gatebox: PIX_GATEWAY_*, SULCRED_OUT_URL (ou outro gateway), MESSAGING_BACKEND=sync"
  fi
else
  ok "(instale jq para validar POST /pix/qrcode)"
fi

if [[ "${FAILED}" -eq 0 ]]; then
  printf '\n[verify] Stack coerente para WhatsApp / comprador B receber PIX Gatebox.\n'
  exit 0
fi
printf '\n[verify] Corrija os itens acima (ver money/.env.example).\n' >&2
exit 1
