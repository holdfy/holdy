# Plano Front-Gatebox

Uma tabela por módulo. Cada linha = uma página. Colunas: **Página**, **APIs desta página**, **Feito** (marque [x] quando implementar).

Base: API Go `/api/v1` (gateboxgo). Projeto: `../front-gatebox`.

---

## Módulo 1 – Autenticação e layout

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Login Customer | POST /customers/auth/login | [ ] |
| Login Admin | POST /admin/auth/login | [ ] |
| Login Backoffice | POST /backoffice/auth/login | [ ] |
| Guard + token + logout | (client-side) | [ ] |

---

## Módulo 2 – Customers

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Dashboard customer | GET /customers/account/balance, GET /customers/pix/transactions | [ ] |
| Conta – Saldo | GET /customers/account/balance | [ ] |
| Conta – Extrato | GET /customers/account/extract | [ ] |
| Conta – Limites | GET /customers/account/limits | [ ] |
| Chaves PIX – lista | GET /customers/account/keys | [ ] |
| Chaves PIX – cadastrar | POST /customers/account/keys | [ ] |
| Chaves PIX – remover | DELETE /customers/account/keys/:id | [ ] |
| PIX – Enviar | POST /customers/pix/send | [ ] |
| PIX – Decodificar BR Code | POST /customers/pix/decode-brcode | [ ] |
| PIX – Gerar QR Code | POST /customers/pix/qrcode | [ ] |
| PIX – Status e histórico | GET /customers/pix/status, GET /customers/pix/transactions | [ ] |
| PIX – Reversão (DPIX) | POST /customers/pix/reversal | [ ] |
| P2P – Enviar | POST /customers/p2p/send | [ ] |
| P2P – Histórico e status | GET /customers/p2p/history, GET /customers/p2p/status/:transfer_id | [ ] |
| P2P – Buscar destinatário | GET /customers/p2p/search | [ ] |

---

## Módulo 3 – Admin

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Dashboard admin | GET /admin/customers, GET /admin/pix/transactions | [ ] |
| Clientes – listar | GET /admin/customers | [ ] |
| Clientes – detalhe | GET /admin/customers/:id | [ ] |
| Clientes – editar | PUT /admin/customers/:id | [ ] |
| Clientes – aprovar KYC | POST /admin/customers/:id/kyc | [ ] |
| PIX – listar transações | GET /admin/pix/transactions | [ ] |
| PIX – detalhe transação | GET /admin/pix/transactions/:id | [ ] |
| PIX – cancelar transação | POST /admin/pix/transactions/:id/cancel | [ ] |
| PIX – criar QR Code | POST /admin/pix/qrcode | [ ] |
| Parceiros – listar | GET /admin/settings/partners | [ ] |
| Parceiros – criar/editar/excluir | POST /admin/settings/partners, PUT /admin/settings/partners/:id, DELETE /admin/settings/partners/:id | [ ] |
| Webhooks – listar | GET /admin/webhooks | [ ] |
| Webhooks – CRUD e testar | POST /admin/webhooks, GET/PUT/DELETE /admin/webhooks/:id, POST /admin/webhooks/:id/test | [ ] |
| Configurações | GET /admin/settings, PUT /admin/settings | [ ] |
| Trocar senha (admin) | POST /admin/auth/change-password | [ ] |

---

## Módulo 4 – Relatórios administrativos

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Relatório de usuários/contas | GET /admin/customers, accounts (legado), status e KYC | [ ] |
| Relatório de MED | sec_med, control_med, history_med | [ ] |
| Relatório de parceiros e webhooks | GET /admin/settings/partners, GET /admin/webhooks | [ ] |
| Dashboard administrativo (métricas) | GET /admin/customers, GET /admin/pix/transactions, sec_med, accounts | [ ] |

---

## Módulo 5 – Relatórios financeiros

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Relatório de saldos | GET /customers/account/balance (por conta), sec_med | [ ] |
| Relatório de transações | GET /admin/pix/transactions, transaction (list + filtros) | [ ] |
| Relatório de taxas (TTO/TPO) | transaction (sub_type, fee_total, fee_fixed, fee_percent_amount) | [ ] |
| Relatório de reversões (DPIX) | transaction (sub_type DPIX) | [ ] |
| Relatório de invoices | invoice (list, invoice_status_id) | [ ] |

---

## Módulo 6 – Relatórios de diferenças / conciliação

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Conciliação PIX (enviadas x confirmadas) | transaction, status_transaction_types | [ ] |
| Diferenças de saldo / inconsistências | accounts, transaction, sec_med | [ ] |
| Pendências (MED, reversões, falhas) | sec_med, transaction (status) | [ ] |

---

## Módulo 7 – Backoffice

| Página | APIs desta página | Feito |
|--------|-------------------|-------|
| Logs | GET /backoffice/logs | [ ] |
| Métricas | GET /backoffice/logs/metrics | [ ] |
| Logs de transações | GET /backoffice/logs/transactions | [ ] |
| Logs de erros | GET /backoffice/logs/errors | [ ] |
| Contas – listar | GET /backoffice/accounts | [ ] |
| Contas – estatísticas | GET /backoffice/accounts/statistics | [ ] |
| Contas – detalhe e transações | GET /backoffice/accounts/:id, GET /backoffice/accounts/:id/transactions | [ ] |
| Contas – atualizar status | PUT /backoffice/accounts/:id/status | [ ] |

---

Legenda: [ ] = não feito | [x] = feito.
