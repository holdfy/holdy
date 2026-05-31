Super prompt (para colar no Cursor do outro projeto)
Você vai integrar este projeto com a API Gatebox Rust (PIX) que roda localmente em http://localhost:8080 (Swagger em http://localhost:8080/swagger/, Basic Auth do Swagger: endpoint / segredo).

Quero uma integração pronta para produção (configurável por env var), com timeouts, retries com backoff, logs estruturados, e idempotência (via externalId). Entregue código + testes.

Contexto do sistema Gatebox Rust (o que já existe)
Base URLs (ambiente local padrão)
API principal (Gatebox): http://localhost:8080
Simulador (não é necessário pra integração): http://localhost:7070
Opção A (recomendada): rotas “Customers” com JWT
Login (gera token Bearer)
POST /api/v1/customers/auth/login
Request JSON:
{ "username": "customer1", "password": "customer1" }
Response JSON:

{ "accessToken": "<jwt>", "tokenType": "Bearer" }
Depois disso, chame os endpoints abaixo com header:

Authorization: Bearer <accessToken>
Content-Type: application/json
Gerar QR Code / “copia e cola” (PIX IN)
POST /api/v1/customers/pix/qrcode
Request JSON (camelCase):
{
  "amount": 10.5,
  "payerName": "Fulano",
  "payerDocument": "12345678900",
  "description": "Pedido 123",
  "expirationSeconds": 1800,
  "reference": "pedido-123",
  "pixKey": "test@simulator.com"
}
Response JSON:

{
  "statusCode": 200,
  "qrCode": "....",          // payload do copia-e-cola
  "txId": "TX....",
  "expiresAt": "RFC3339",
  "transactionId": "TX....",
  "gateway": "sulcred",
  "data": { "...": "..." }
}
Enviar pagamento (PIX OUT) por chave Pix
POST /api/v1/customers/pix/send
Request JSON (camelCase):
{
  "account": "2000001",
  "bank": "00000000",
  "branch": "0001",
  "documentNumber": "12345678900",
  "name": "Destinatário",
  "amount": 1.0,
  "key": "destinatario@email.com",
  "typeKey": "EMAIL",
  "externalId": "pedido-123"
}
Response JSON:

{
  "statusCode": 200,
  "transactionId": "4005",
  "data": { "status": "NEW", "type": "DEBIT", "rate": 0.1, "amount": 0.9, "...": "..." }
}
Observação: o processamento do PIX OUT é assíncrono; o retorno NEW significa “criado e enfileirado”.

Opção B (alternativa): rotas “core pix” sem JWT (menos segura)
POST /api/v1/pix/qrcode (snake_case no request)
POST /api/v1/pix/send (camelCase e precisa userId no body)
Use apenas se a arquitetura do outro projeto exigir.

Requisito de negócio da integração (o que eu preciso que você implemente)
1) “Documentação”/cliente para Gerar QR Code (copia-e-cola)
Função/serviço CreatePixQrCode(...) que:
chama POST /api/v1/customers/pix/qrcode
retorna um objeto com: qrCode, txId, expiresAt, gateway
valida amount > 0
aplica timeout (ex.: 10s) e retries somente para falhas transitórias (5xx, timeout, conexão)
2) “Documentação”/cliente para Pagar
Função/serviço PayPixByKey(...) que:
chama POST /api/v1/customers/pix/send
usa externalId obrigatório (idempotência do integrador)
valida amount > 0, key != "", documentNumber presente, etc.
retorna transactionId e o status retornado
3) Caso de uso: “copia e cola para pagamento”
Hoje o Gatebox Rust não tem um endpoint estável “pagar BR Code direto” (o decode decode-brcode existe no path customers mas está stub no momento). Então implemente uma destas estratégias (escolha e documente no PR):

Estratégia 1 (preferida): se o outro sistema já fornece key + amount, pague via PayPixByKey.
Estratégia 2 (se o input for BR Code): implementar um decoder de BR Code no outro projeto para extrair os campos necessários (quando possível) e então pagar via PayPixByKey. Se não for possível extrair, falhe com erro claro “unsupported brcode”.
Estratégia 3 (se você puder alterar o Gatebox): abrir um PR separado no gatebox-rust implementando /customers/pix/decode-brcode e/ou /customers/pix/pay-brcode. (Só proponha; não implemente aqui no outro projeto.)
Configuração (env vars do outro projeto)
Crie estas env vars no outro projeto e use defaults seguros:

GATEBOX_BASE_URL (default: http://localhost:8080)
GATEBOX_TIMEOUT_MS (default: 10000)
GATEBOX_RETRY_MAX (default: 2)
GATEBOX_CUSTOMER_USERNAME
GATEBOX_CUSTOMER_PASSWORD
A autenticação deve:

fazer login e cachear token (com expiração), re-logar quando 401/expired.
Critérios de aceite (obrigatórios)
QR Code: uma chamada real retorna statusCode=200 e um qrCode não vazio.
Pay: uma chamada real retorna statusCode=200 e transactionId preenchido.
Idempotência: duas chamadas iguais com o mesmo externalId não podem gerar duplicidade (o integrador deve tratar erro/duplicate de forma previsível).
Observabilidade: logs incluem externalId, txId, transactionId, latência e status HTTP (sem vazar secrets).
Testes:
testes unitários com mock do HTTP
pelo menos 1 teste de integração opcional (feature flag) que roda contra http://localhost:8080
O que eu quero como entrega do Cursor
Um módulo “GateboxClient”/“PixGatewayClient” com:
Authenticate()
CreatePixQrCode()
PayPixByKey()
(opcional) PayPixByCopyPaste() implementando a estratégia escolhida
Tipos/DTOs de request/response coerentes com os JSON acima
README curto com exemplos de uso e exemplos de curl
Exemplos de chamadas (para você usar em testes manuais)
Login
curl -sS -X POST "http://localhost:8080/api/v1/customers/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"customer1","password":"customer1"}'
QR Code (com token)
curl -sS -X POST "http://localhost:8080/api/v1/customers/pix/qrcode" \
  -H "Authorization: Bearer <TOKEN>" -H "Content-Type: application/json" \
  -d '{"amount":10.5,"payerName":"Fulano","payerDocument":"12345678900","description":"Pedido 123","expirationSeconds":1800,"reference":"pedido-123","pixKey":"test@simulator.com"}'
Pagar por chave (com token)
curl -sS -X POST "http://localhost:8080/api/v1/customers/pix/send" \
  -H "Authorization: Bearer <TOKEN>" -H "Content-Type: application/json" \
  -d '{"account":"2000001","bank":"00000000","branch":"0001","documentNumber":"12345678900","name":"Destinatário","amount":1.0,"key":"destinatario@email.com","typeKey":"EMAIL","externalId":"pedido-123"}'
Se você me disser qual linguagem/stack é o “outro projeto” (Go, Node, Java, Python), eu adapto esse super prompt para pedir implementação idiomática (ex.: client HTTP com middlewares, circuit breaker, etc.) e já com esqueleto de testes na ferramenta certa.