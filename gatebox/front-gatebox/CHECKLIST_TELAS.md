# Checklist – Telas Front-Gatebox

Atualize este arquivo marcando com `[x]` conforme for implementando. Base: plano front-gatebox + regras_de_negocio.md (gateboxgo).

**Última atualização:** 2025-03-06

---

## Lista de telas a criar (visão geral)

Marque `[x]` à medida que cada tela for criada.

### Setup e autenticação
- [x] Setup do projeto (env, client HTTP, rotas por perfil)
- [x] Login Customer
- [x] Login Admin
- [x] Login Backoffice
- [x] Guard de rotas + token + logout

### Área do cliente (Customers)
- [x] Dashboard customer
- [x] Conta – Saldo (balance, MED, disponível)
- [x] Conta – Extrato
- [x] Conta – Limites
- [x] Chaves PIX – listar / cadastrar / remover
- [x] PIX – Enviar
- [x] PIX – Decodificar BR Code
- [x] PIX – Gerar QR Code
- [x] PIX – Status e histórico
- [x] PIX – Status da invoice
- [x] PIX – Reversão (DPIX)
- [x] P2P – Enviar / Histórico
- [x] Webhooks do cliente

### Área Admin
- [x] Dashboard admin
- [x] Clientes – listar / detalhe / KYC
- [x] Clientes – saldo por cliente / extrato por cliente
- [x] Clientes – criar conta / lucro (get-profit)
- [x] PIX – listar transações / detalhe / cancelar / QR Code / status por end-to-end
- [x] Conta admin – saldo / extrato / total MED / contagem PIX in-out / comprovante
- [x] MED – listar / detalhe
- [x] Key PIX (admin) – listar / cadastrar
- [x] Parceiros – listar / CRUD
- [x] Webhooks (admin) – CRUD
- [x] Settings admin
- [x] Admin – trocar senha

### Relatórios administrativos
- [x] Relatório de usuários/contas (listagem, status, KYC)
- [x] Relatório de atividades por cliente
- [x] Relatório de MED (listagem, status, prazos)
- [x] Relatório de parceiros e uso por gateway
- [x] Relatório de webhooks (configurações, testes, falhas)
- [x] Relatório de logs/auditoria (ações admin)
- [x] Dashboard administrativo (métricas consolidadas)

### Relatórios financeiros
- [x] Relatório de saldos (por conta, total MED bloqueado)
- [x] Relatório de transações (PIX in/out, período, tipo)
- [x] Relatório de taxas (TTO, TPO, por período ou por conta)
- [x] Relatório de lucro (get-profit / receita da plataforma)
- [x] Relatório de extrato consolidado (admin)
- [x] Relatório de reversões (DPIX) realizadas
- [x] Relatório de invoices (QR Code – criadas, pagas, canceladas)

### Relatórios de diferenças / conciliação
- [x] Diferenças por transação (valor esperado x realizado, status)
- [x] Conciliação PIX (transações enviadas x confirmadas pelo gateway)
- [x] Conciliação de taxas (calculado x cobrado / auditável)
- [x] Diferenças de saldo (saldo calculado x saldo exibido / ajustes)
- [x] Relatório de inconsistências ou pendências (MED, reversões, falhas)

### Área Backoffice _(opcional)_
- [x] Login backoffice
- [x] Logs e métricas
- [x] Contas (listar, estatísticas, transações por conta)

### Regras de negócio e polimento
- [x] Comportamento das telas conforme regras_de_negocio.md (saldo, MED, mensagens)
- [x] Tratamento de erros (SnackbarContext em webhooks, etc.)

---

## Setup do projeto

- [ ] Projeto criado em `../front-gatebox` (baseado em front_pro)
- [ ] `package.json` com nome `front-gatebox`
- [ ] Variável `REACT_APP_API_BASE_URL` (ex.: `.env` / `.env.example`)
- [ ] Client HTTP (axios/fetch) com base URL e header `Authorization: Bearer`
- [ ] Rotas por perfil (customer / admin / backoffice) após login

---

## Autenticação

- [ ] **Login Customer** – tela de login (`POST /api/v1/customers/auth/login`)
- [ ] **Login Admin** – tela de login (`POST /api/v1/admin/auth/login`)
- [ ] **Login Backoffice** – tela de login (`POST /api/v1/backoffice/auth/login`) _(opcional)_
- [ ] Guard de rotas: redirecionar para login se não autenticado
- [ ] Armazenar token (localStorage/sessionStorage) e enviar no header
- [ ] Logout: limpar token e redirecionar para login

---

## Área do cliente (Customers)

### Dashboard
- [ ] Dashboard customer: resumo (saldo, últimas transações, atalhos)

### Conta
- [ ] **Saldo** – exibir `balance`, `preventiveBlock` (MED), `availableBalance` (regra: três valores)
- [ ] **Extrato** – lista de transações (`/account/extract`)
- [ ] **Limites** – tela/consulta (`/account/limits`)
- [ ] Exibir sub-tipos legíveis (PIX, DPIX, TTO, TPO, SMD) no extrato/detalhe

### Chaves PIX
- [ ] Listar chaves (`GET /account/keys`)
- [ ] Cadastrar chave (`POST /account/keys`)
- [ ] Remover chave (`DELETE /account/keys/:id`)

### PIX
- [ ] **Enviar PIX** – formulário e chamada `POST /pix/send`
- [ ] Prévia de taxa / total debitado (quando API permitir)
- [ ] Mensagem de saldo insuficiente (balance, taxa, valor) conforme regra
- [ ] Mensagem de idempotência (duplicate externalId) conforme regra
- [ ] **Decodificar BR Code** – tela e `POST /pix/decode-brcode`
- [ ] **Gerar QR Code** – tela e `POST /pix/qrcode` (create-immediate-qrcode)
- [ ] **Status da invoice** – exibir CREATED/DONE/CANCEL; exceção tipo FIXED
- [ ] **Status e histórico PIX** – `GET /pix/status`, `GET /pix/transactions`
- [ ] **Reversão (DPIX)** – formulário e `POST /pix/reversal`; valor ≤ original; mensagens de erro da API

### P2P _(se for requisito)_
- [ ] Enviar P2P
- [ ] Histórico e status P2P

### Webhook (customer)
- [ ] Listar/cadastrar webhooks do cliente (`/customers/webhook-manager`) _(se existir na API)_

---

## Área Admin

### Dashboard
- [ ] Dashboard admin: métricas e atalhos

### Clientes
- [ ] Listar clientes (`GET /admin/customers`)
- [ ] Ver detalhe do cliente (`GET /admin/customers/:id`)
- [ ] Editar cliente (`PUT /admin/customers/:id`)
- [ ] Aprovar KYC (`POST /admin/customers/:id/kyc`)
- [ ] Saldo por cliente (`get-balance-customers/:uuid` / equivalente Go)
- [ ] Extrato por cliente (equivalente Go)
- [ ] Criar conta cliente (equivalente `create-custumers-account`)
- [ ] Lucro (`get-profit` / equivalente Go)

### PIX (admin)
- [ ] Listar transações PIX
- [ ] Detalhe da transação
- [ ] Cancelar transação (`POST /admin/pix/transactions/:id/cancel`)
- [ ] Criar QR Code (admin)
- [ ] Status por end-to-end (equivalente `status-pix-by-endtoend`)

### Conta (admin)
- [ ] Saldo admin (`/admin/account` ou equivalente)
- [ ] Extrato admin
- [ ] Total MED (`total-med` / equivalente)
- [ ] Contagem PIX in/out (`count-pix-in-out` / equivalente)
- [ ] Comprovante por transação (`transaction-for-receipt` / equivalente)

### MED (admin)
- [ ] Listar MEDs (`/api/v1/sec_med/*` ou endpoints admin equivalentes)
- [ ] Detalhe do MED
- [ ] Responder MED (`response-med`)
- [ ] Descontar MEDs (`discount-meds`)

### Key PIX (admin)
- [ ] Listar chaves PIX (admin)
- [ ] Cadastrar chave PIX (admin)

### Parceiros e configurações
- [ ] Listar parceiros (`GET /admin/settings/partners`)
- [ ] Criar/editar/remover parceiro
- [ ] Webhooks (CRUD) (`/admin/webhooks`)
- [ ] Settings (styled-settings ou equivalente)

### Auth (admin)
- [ ] Trocar senha (`changer-pwd` / `change-password`)

---

## Área Backoffice _(opcional)_

- [ ] Login backoffice
- [ ] Logs (`/backoffice/logs`)
- [ ] Métricas (`/backoffice/logs/metrics`)
- [ ] Contas: listar, estatísticas, transações por conta (`/backoffice/accounts`)

---

## Relatórios administrativos

- [ ] **Relatório de usuários/contas** – listagem de contas, status (ACTIVE, PENDING_KYC, etc.), KYC pendente
- [ ] **Relatório de atividades por cliente** – ações, acessos, transações por cliente
- [ ] **Relatório de MED** – listagem de MEDs, status (OPEN, RETURNED, etc.), prazos (scheduled_date), valores
- [ ] **Relatório de parceiros e uso** – parceiros configurados, uso por gateway, volume
- [ ] **Relatório de webhooks** – configurações, histórico de testes, falhas ou retentativas
- [ ] **Relatório de logs/auditoria** – ações de admin (quem, quando, o quê)
- [ ] **Dashboard administrativo** – métricas consolidadas (contas ativas, transações do dia, MED total, etc.)

---

## Relatórios financeiros

- [ ] **Relatório de saldos** – saldo por conta, total em MED bloqueado, saldo disponível
- [ ] **Relatório de transações** – PIX in/out por período, filtros por tipo (PIX, DPIX, TTO, TPO, SMD)
- [ ] **Relatório de taxas** – TTO/TPO por período ou por conta (receita da plataforma x comissão parceiro)
- [ ] **Relatório de lucro** – receita da plataforma (get-profit ou equivalente na API)
- [ ] **Extrato consolidado (admin)** – todas as transações com filtros (conta, data, tipo)
- [ ] **Relatório de reversões (DPIX)** – reversões realizadas, valor, taxa, status
- [ ] **Relatório de invoices** – QR Code criados, pagos (DONE), cancelados, por período

---

## Relatórios de diferenças / conciliação

- [ ] **Diferenças por transação** – valor esperado x valor realizado, status (ex.: processando x concluído x falha)
- [ ] **Conciliação PIX** – transações enviadas x confirmadas pelo gateway (end-to-end, status)
- [ ] **Conciliação de taxas** – valor calculado (fee_total, TTO, TPO) x valor efetivo / auditável
- [ ] **Diferenças de saldo** – saldo calculado x saldo exibido; ajustes ou inconsistências
- [ ] **Relatório de inconsistências/pendências** – MED em aberto, reversões pendentes, transações com falha para análise

---

## Regras de negócio (comportamento nas telas)

- [ ] **Saldo:** sempre exibir balance + preventiveBlock + availableBalance onde aplicável
- [ ] **Status conta:** mensagem clara em erro (PENDING_KYC, invalid status); link KYC se possível
- [ ] **PIX OUT:** mensagens de saldo insuficiente e idempotência conforme regras_de_negocio.md
- [ ] **Reversão:** valor ≤ original; taxa de reversal; mensagens da API
- [ ] **MED:** exibir bloqueio e data de liberação (90 dias) quando a API expuser
- [ ] **Invoice:** status CREATED/DONE/CANCEL; exceção FIXED
- [ ] **Taxas:** exibir resumo (fixo + percentual) quando a API retornar
- [ ] **Sub-tipos:** PIX, Devolução, Taxa operacional, Taxa parceiro, MED em extrato/detalhe
- [ ] **Account rules / Whitelist:** mensagem compreensível quando API rejeitar

---

## Polimento

- [ ] Tratamento de erros da API (toast/snackbar ou alertas)
- [ ] CORS configurado no backend para origem do front em desenvolvimento
- [ ] Testes manuais contra a API Go (gateboxgo)

---

## Legenda

- `[ ]` – não feito  
- `[x]` – feito  
- _(opcional)_ – pode ser deixado para depois
