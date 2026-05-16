# Lista de telas, APIs por módulo e gráficos – Front-Gatebox

Documento de referência para implementação: telas a criar, APIs utilizadas por módulo e gráficos para operação financeira. Marque `[x]` quando cada item for implementado.

**Base:** API Go em `GET/POST .../api/v1/...` (gateboxgo).  
**Última atualização:** _(preencher ao atualizar)_

---

## 1. APIs por módulo

Todas as requisições utilizam o prefixo base: `/api/v1`.

### 1.1 Módulo Customers (`/api/v1/customers`)

| Método | Endpoint | Descrição |
|--------|----------|------------|
| POST   | `/customers/auth/login` | Login |
| POST   | `/customers/auth/register` | Registro |
| GET    | `/customers/auth/profile` | Perfil (protegido) |
| PUT    | `/customers/auth/profile` | Atualizar perfil |
| POST   | `/customers/auth/change-password` | Trocar senha |
| GET    | `/customers/account/balance` | Saldo (balance, preventiveBlock, availableBalance) |
| GET    | `/customers/account/extract` | Extrato |
| GET    | `/customers/account/limits` | Limites |
| GET    | `/customers/account/keys` | Listar chaves PIX |
| POST   | `/customers/account/keys` | Cadastrar chave PIX |
| DELETE | `/customers/account/keys/:id` | Remover chave PIX |
| POST   | `/customers/pix/send` | Enviar PIX |
| POST   | `/customers/pix/decode-brcode` | Decodificar BR Code |
| POST   | `/customers/pix/qrcode` | Gerar QR Code |
| GET    | `/customers/pix/status` | Status da transação |
| GET    | `/customers/pix/transactions` | Listar transações PIX |
| POST   | `/customers/pix/reversal` | Reversão (DPIX) |
| POST   | `/customers/p2p/send` | Enviar P2P |
| GET    | `/customers/p2p/history` | Histórico P2P |
| GET    | `/customers/p2p/status/:transfer_id` | Status P2P |
| GET    | `/customers/p2p/search` | Buscar destinatário |

**Checklist implementação – Customers**

- [ ] Auth: login, register, profile, change-password
- [ ] Account: balance, extract, limits, keys (CRUD)
- [ ] PIX: send, decode-brcode, qrcode, status, transactions, reversal
- [ ] P2P: send, history, status, search

---

### 1.2 Módulo Admin (`/api/v1/admin`)

| Método | Endpoint | Descrição |
|--------|----------|------------|
| POST   | `/admin/auth/login` | Login admin |
| GET    | `/admin/auth/profile` | Perfil admin |
| POST   | `/admin/auth/change-password` | Trocar senha |
| GET    | `/admin/customers` | Listar clientes |
| GET    | `/admin/customers/:id` | Detalhe do cliente |
| PUT    | `/admin/customers/:id` | Atualizar cliente |
| DELETE | `/admin/customers/:id` | Excluir cliente |
| POST   | `/admin/customers/:id/kyc` | Aprovar KYC |
| GET    | `/admin/pix/transactions` | Listar transações PIX |
| GET    | `/admin/pix/transactions/:id` | Detalhe da transação |
| POST   | `/admin/pix/send` | Enviar PIX (admin) |
| GET    | `/admin/pix/status` | Status PIX |
| POST   | `/admin/pix/qrcode` | Criar QR Code (admin) |
| POST   | `/admin/pix/transactions/:id/cancel` | Cancelar transação |
| GET    | `/admin/settings` | Obter configurações |
| PUT    | `/admin/settings` | Atualizar configurações |
| GET    | `/admin/settings/partners` | Listar parceiros |
| POST   | `/admin/settings/partners` | Criar parceiro |
| PUT    | `/admin/settings/partners/:id` | Atualizar parceiro |
| DELETE | `/admin/settings/partners/:id` | Excluir parceiro |
| GET    | `/admin/webhooks` | Listar webhooks |
| POST   | `/admin/webhooks` | Criar webhook |
| GET    | `/admin/webhooks/:id` | Detalhe webhook |
| PUT    | `/admin/webhooks/:id` | Atualizar webhook |
| DELETE | `/admin/webhooks/:id` | Excluir webhook |
| POST   | `/admin/webhooks/:id/test` | Testar webhook |

**Checklist implementação – Admin**

- [ ] Auth: login, profile, change-password
- [ ] Customers: list, get, update, delete, approve KYC
- [ ] PIX: list transactions, get, send, status, qrcode, cancel
- [ ] Settings: get, update, partners CRUD
- [ ] Webhooks: CRUD, test

---

### 1.3 Módulo Backoffice (`/api/v1/backoffice`)

| Método | Endpoint | Descrição |
|--------|----------|------------|
| POST   | `/backoffice/auth/login` | Login backoffice |
| GET    | `/backoffice/auth/profile` | Perfil |
| GET    | `/backoffice/logs` | Logs |
| GET    | `/backoffice/logs/metrics` | Métricas |
| GET    | `/backoffice/logs/transactions` | Logs de transações |
| GET    | `/backoffice/logs/errors` | Logs de erros |
| GET    | `/backoffice/accounts` | Listar contas |
| GET    | `/backoffice/accounts/statistics` | Estatísticas de contas |
| GET    | `/backoffice/accounts/:id` | Detalhe da conta |
| GET    | `/backoffice/accounts/:id/transactions` | Transações da conta |
| PUT    | `/backoffice/accounts/:id/status` | Atualizar status da conta |

**Checklist implementação – Backoffice**

- [ ] Auth: login, profile
- [ ] Logs: list, metrics, transactions, errors
- [ ] Accounts: list, statistics, get, transactions, update status

---

### 1.4 Endpoints legados (dados para relatórios e gráficos)

Utilizados para montar relatórios e gráficos (admin/backoffice). Prefixo: `/api/v1`.

| Recurso | Endpoints típicos | Uso |
|---------|-------------------|-----|
| Transações | `transaction` (GET list, filtros) | Extrato consolidado, relatórios financeiros |
| MED | `sec_med`, `control_med`, `history_med` | Relatórios MED, gráfico MED bloqueado |
| Invoice | `invoice` (GET list, por status) | Relatório de invoices, gráfico criadas vs pagas |
| Parceiros | `partners` | Relatório de parceiros |
| Contas | `accounts` | Relatório de contas, saldos |
| Tipos | `status_transaction_types`, `sub_type_transaction_types` | Legendas em gráficos e tabelas |

---

## 2. Uma tabela por módulo (cada linha = uma página; colunas = APIs e checkbox)

Cada módulo tem uma tabela. Em cada tabela: **uma linha = uma página**; colunas **APIs** (lista de endpoints) e **Checkbox** (marque `[x]` quando implementado).

### 2.1 Módulo Autenticação e layout

| Página | APIs | Checkbox |
|--------|------|----------|
| Login Customer | POST /customers/auth/login | [ ] |
| Login Admin | POST /admin/auth/login | [ ] |
| Login Backoffice | POST /backoffice/auth/login | [ ] |
| Guard de rotas + token + logout | (client-side; rotas protegidas) | [ ] |

---

### 2.2 Módulo Customers

| Página | APIs | Checkbox |
|--------|------|----------|
| Dashboard customer | GET /customers/account/balance; GET /customers/pix/transactions | [ ] |
| Conta – Saldo | GET /customers/account/balance | [ ] |
| Conta – Extrato | GET /customers/account/extract | [ ] |
| Conta – Limites | GET /customers/account/limits | [ ] |
| Chaves PIX – lista | GET /customers/account/keys | [ ] |
| Chaves PIX – cadastrar | POST /customers/account/keys | [ ] |
| Chaves PIX – remover | DELETE /customers/account/keys/:id | [ ] |
| PIX – Enviar | POST /customers/pix/send | [ ] |
| PIX – Decodificar BR Code | POST /customers/pix/decode-brcode | [ ] |
| PIX – Gerar QR Code | POST /customers/pix/qrcode | [ ] |
| PIX – Status e histórico | GET /customers/pix/status; GET /customers/pix/transactions | [ ] |
| PIX – Reversão (DPIX) | POST /customers/pix/reversal | [ ] |
| P2P – Enviar | POST /customers/p2p/send | [ ] |
| P2P – Histórico e status | GET /customers/p2p/history; GET /customers/p2p/status/:transfer_id | [ ] |
| P2P – Buscar destinatário | GET /customers/p2p/search | [ ] |

---

### 2.3 Módulo Admin

| Página | APIs | Checkbox |
|--------|------|----------|
| Dashboard admin | GET /admin/customers; GET /admin/pix/transactions | [ ] |
| Clientes – listar | GET /admin/customers | [ ] |
| Clientes – detalhe | GET /admin/customers/:id | [ ] |
| Clientes – editar | PUT /admin/customers/:id | [ ] |
| Clientes – aprovar KYC | POST /admin/customers/:id/kyc | [ ] |
| PIX – listar transações | GET /admin/pix/transactions | [ ] |
| PIX – detalhe transação | GET /admin/pix/transactions/:id | [ ] |
| PIX – cancelar transação | POST /admin/pix/transactions/:id/cancel | [ ] |
| PIX – criar QR Code | POST /admin/pix/qrcode | [ ] |
| Parceiros – listar | GET /admin/settings/partners | [ ] |
| Parceiros – criar / editar / excluir | POST /admin/settings/partners; PUT /admin/settings/partners/:id; DELETE /admin/settings/partners/:id | [ ] |
| Webhooks – listar | GET /admin/webhooks | [ ] |
| Webhooks – criar / editar / excluir / testar | POST /admin/webhooks; GET /admin/webhooks/:id; PUT /admin/webhooks/:id; DELETE /admin/webhooks/:id; POST /admin/webhooks/:id/test | [ ] |
| Configurações | GET /admin/settings; PUT /admin/settings | [ ] |
| Trocar senha (admin) | POST /admin/auth/change-password | [ ] |

---

### 2.4 Módulo Relatórios administrativos

| Página | APIs | Checkbox |
|--------|------|----------|
| Relatório de usuários/contas | GET /admin/customers; accounts (legado); status e KYC | [ ] |
| Relatório de MED | GET /api/v1/sec_med/*; control_med; history_med | [ ] |
| Relatório de parceiros e webhooks | GET /admin/settings/partners; GET /admin/webhooks | [ ] |
| Dashboard administrativo (métricas) | GET /admin/customers; GET /admin/pix/transactions; sec_med; accounts | [ ] |

---

### 2.5 Módulo Relatórios financeiros

| Página | APIs | Checkbox |
|--------|------|----------|
| Relatório de saldos | GET /customers/account/balance (por conta); sec_med (MED bloqueado) | [ ] |
| Relatório de transações | GET /admin/pix/transactions; transaction (list + filtros) | [ ] |
| Relatório de taxas (TTO/TPO) | transaction (sub_type; fee_total; fee_fixed; fee_percent_amount) | [ ] |
| Relatório de reversões (DPIX) | transaction (sub_type DPIX) | [ ] |
| Relatório de invoices | invoice (list; invoice_status_id) | [ ] |

---

### 2.6 Módulo Relatórios de diferenças / conciliação

| Página | APIs | Checkbox |
|--------|------|----------|
| Conciliação PIX (enviadas x confirmadas) | transaction; status_transaction_types | [ ] |
| Diferenças de saldo / inconsistências | accounts; transaction; sec_med | [ ] |
| Pendências (MED, reversões, falhas) | sec_med; transaction (status) | [ ] |

---

### 2.7 Módulo Backoffice

| Página | APIs | Checkbox |
|--------|------|----------|
| Logs | GET /backoffice/logs | [ ] |
| Métricas | GET /backoffice/logs/metrics | [ ] |
| Logs de transações | GET /backoffice/logs/transactions | [ ] |
| Logs de erros | GET /backoffice/logs/errors | [ ] |
| Contas – listar | GET /backoffice/accounts | [ ] |
| Contas – estatísticas | GET /backoffice/accounts/statistics | [ ] |
| Contas – detalhe e transações | GET /backoffice/accounts/:id; GET /backoffice/accounts/:id/transactions | [ ] |
| Contas – atualizar status | PUT /backoffice/accounts/:id/status | [ ] |

---

## 3. Gráficos para operação financeira

Gráficos reais utilizáveis em operação financeira: tipo, fonte de dados e tela sugerida.

### 3.1 Volume e evolução

| Gráfico | Tipo | Fonte de dados | Tela sugerida |
|---------|------|----------------|---------------|
| Volume de transações no tempo | Linha ou barras (agregado por dia/semana/mês) | GET pix/transactions ou transaction (list) com filtro de data; agrupar por período | Dashboard admin, Relatório financeiro |
| PIX entrada x PIX saída por período | Barras agrupadas (entrada vs saída) | transaction (type CREDIT vs DEBIT, sub_type PIX) por período | Dashboard admin, Relatório financeiro |
| Evolução do saldo (conta ou consolidado) | Linha no tempo | account/balance ao longo do tempo ou snapshot; ou soma de transaction por data | Dashboard customer/admin, Relatório de saldos |
| MED bloqueado no tempo | Área ou linha | sec_med (amount, status OPEN) agregado por data | Dashboard admin, Relatório MED |

### 3.2 Receita e taxas

| Gráfico | Tipo | Fonte de dados | Tela sugerida |
|---------|------|----------------|---------------|
| Receita por taxas (TTO/TPO) no tempo | Barras ou área empilhada | transaction (fee_total, sub_type TTO/TPO) por período | Relatório financeiro / taxas |
| Distribuição por tipo de transação | Pizza ou barras (PIX, DPIX, TTO, TPO, SMD) | transaction agrupado por sub_type_transaction_id | Dashboard admin, Relatório financeiro |
| Taxa média e total por período | Card/KPI + linha | transaction (fee_total, fee_fixed, fee_percent_amount) agregado | Relatório de taxas |

### 3.3 Operacional e conciliação

| Gráfico | Tipo | Fonte de dados | Tela sugerida |
|---------|------|----------------|---------------|
| Status de transações (completada, falha, processando) | Pizza ou barras | transaction agrupado por status_transaction_id | Dashboard admin, Conciliação |
| Invoices: criadas x pagas x canceladas | Barras ou funil | invoice agrupado por invoice_status_id | Relatório de invoices |
| Conciliação: valor esperado x realizado | Barras comparativas (lado a lado) | transaction (requested_amount vs amount, status) | Relatório de diferenças |
| Top N contas por volume | Barras horizontais | transaction agregado por account_id, ordenar por volume | Relatório financeiro / admin |

### 3.4 Lista de gráficos – checklist implementação

- [ ] Volume de transações no tempo (linha/barras)
- [ ] PIX entrada x saída por período (barras agrupadas)
- [ ] Evolução do saldo (linha)
- [ ] MED bloqueado no tempo (área/linha)
- [ ] Receita por taxas TTO/TPO no tempo (barras/área)
- [ ] Distribuição por tipo de transação (pizza/barras)
- [ ] Taxa média e total por período (KPI + linha)
- [ ] Status de transações (pizza/barras)
- [ ] Invoices criadas x pagas x canceladas (barras/funil)
- [ ] Conciliação esperado x realizado (barras comparativas)
- [ ] Top contas por volume (barras horizontais)

---

## 4. Legenda

- `[ ]` – não implementado  
- `[x]` – implementado  
- APIs: sempre com prefixo base `REACT_APP_API_BASE_URL` (ex.: `http://localhost:8081/api/v1`).  
- Gráficos: usar dados das APIs acima; agregar no front (por período, tipo, status) ou via endpoints de relatório se o backend disponibilizar.
