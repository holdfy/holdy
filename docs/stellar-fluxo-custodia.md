# Fluxo de Custódia com Stellar

O comprador paga via PIX. O BlindPay converte o valor fiat em tokens on-chain (BRLx/USDB) e os deposita na carteira Stellar da Holdy (a plataforma opera carteiras próprias — o cliente final não tem carteira Stellar, interage apenas via PIX). O APICash então invoca o smart contract de escrow na Soroban (Stellar) chamando `lock()`, que bloqueia os tokens no contrato vinculados ao `order_id`, ao endereço do comprador e ao do vendedor — ninguém consegue mover os fundos enquanto estão travados.

Quando o comprador confirma que recebeu o produto, ele assina `confirm_delivery()` on-chain. Na sequência, `release()` é invocado: o contrato transfere o principal ao vendedor e distribui o yield acumulado (70% vendedor / 10% comprador / 20% plataforma). O BlindPay então faz o off-ramp, convertendo os tokens de volta para BRL e enviando via PIX ao vendedor.

Se houver disputa, qualquer parte chama `open_dispute()`, os fundos ficam travados no contrato até o admin resolver via `resolve_dispute()` — liberando ao vendedor ou reembolsando o comprador.

**O contrato é a garantia:** nem comprador, nem vendedor, nem a plataforma consegue mover os fundos fora dessas regras codificadas no Wasm deployado na Stellar.
