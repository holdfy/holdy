#!/usr/bin/env bash
# Simula payload **Meta Cloud API** no POST /webhook/whatsapp — útil para CI / regressão do
# handler até o bridge **whatsapp-rust** → WhatsAppEvent estar ligado (transporte acordado).
# Requer: apicash-core :3000 + apicash-whatsapp (webhook, ex. :3010).
set -euo pipefail

WA_BASE="${WA_BASE:-http://127.0.0.1:3010}"
PEER="${WA_TEST_PEER:-5511999887766}"

send() {
  local body="$1"
  local mid="wamid.sim.$RANDOM.$SECONDS.$(date +%s%N)"
  curl -sS -X POST "$WA_BASE/webhook/whatsapp" \
    -H 'Content-Type: application/json' \
    -d "$(jq -n \
      --arg from "$PEER" \
      --arg mid "$mid" \
      --arg text "$body" \
      '{
        object: "whatsapp_business_account",
        entry: [{
          id: "WABA_SIM",
          changes: [{
            field: "messages",
            value: {
              messaging_product: "whatsapp",
              metadata: {
                display_phone_number: "15550000000",
                phone_number_id: "123456789"
              },
              messages: [{
                from: $from,
                id: $mid,
                timestamp: "1700000000",
                type: "text",
                text: { body: $text }
              }]
            }
          }]
        }]
      }')" -o /dev/null -w "POST msg %{http_code}\n"
}

echo "Simulando peer=$PEER contra $WA_BASE (aguarde ~1s entre passos)"
send "novo pedido"
sleep 1
send "10,00"
sleep 1
send "Compra teste via webhook simulado"
sleep 1
send "529.982.247-25"
sleep 1
# Link social melhora o score (evita bloqueio antifraude vs. só "pular")
send "https://instagram.com/apicashdemo"
sleep 1
send "CONFIRMAR_PEDIDO"
sleep 2
send "ja paguei"
sleep 2
send "confirmar recebimento"
sleep 1
echo "Feito. Veja os logs do processo apicash-whatsapp (stub: linhas 'whatsapp stub: text') e do apicash-core."
