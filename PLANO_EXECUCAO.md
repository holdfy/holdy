# Plano de Execução — Holdfy (Arquitetura Sênior)

> **Para qualquer Claude ou IA que abrir este repositório:**
> Este documento é o plano de desenvolvimento oficial do projeto Holdfy/pos-nearx.
> Leia o `CLAUDE.md` primeiro para entender a arquitetura, depois siga este plano na ordem indicada.
> Atualize o status de cada fase conforme for executando (substitua `[ ]` por `[x]`).

---

## Contexto
Monorepo com três stacks paralelas (APICash Rust, Gatebox Rust, site/ React) que precisam convergir para um produto coeso. O código core está mais avançado do que parece — o problema é integração, não implementação from scratch. Leia `ajustes.txt` na raiz para entender a visão completa do produto.

---

## Estado Real do Projeto (snapshot 2026-05-24)

| Componente | Estado | Gap principal |
|---|---|---|
| APICash — escrow/PIX/yield | ✅ 85% funcional | Postgres desativado por padrão, Soroban mock |
| APICash — antifraude (11 fatores, score 0-1000) | ✅ completo | SEFAZ real pendente |
| APICash — WhatsApp bot conversacional | ✅ 70% | Falta bridge multi-device event loop |
| APICash — admin-backend (porta 3001) | 🟡 estrutura OK | Handlers com lógica incompleta |
| APICash — frontend Leptos SSR (porta 3002) | 🟡 template OK | Fetch/integração parcial |
| Gatebox — PIX gateway (porta 8081) | ✅ 85% | Disputas (0%), testes (0%) |
| Gatebox — front-gatebox (React, ~90% telas) | ✅ 90% telas | Gráficos, refinamento |
| site/ — marketplace React (Vite+shadcn+Tailwind) | 🟡 UI completa | Backend: 0% integrado (tudo mock) |
| front-template/ — Material Dashboard 3 PRO | ✅ pronto p/ usar | Aguardando admin Holdfy ser criado |
| PF/PJ — antifraude valida CPF e CNPJ | ✅ completo | Score diferenciado: CNPJ ATIVA=+100, BAIXADA=-200, idade empresa |
| PF/PJ — Gatebox modelos (type_person_types) | ✅ presente | Seeds: NATURAL_PERSON(1), LEGAL_PERSON(2) |
| PF/PJ — Gateway Next | ✅ corrigido | `infer_person_type()` por tamanho do documento (CNPJ=14 dígitos) |
| PF/PJ — Auth/JWT claims | ✅ implementado | `PersonType` no JWT; `document` propagado; `APICASH_AUTH_USERS` aceita 4o campo |
| PF/PJ — WhatsApp | ✅ implementado | `AwaitingBuyerDocument` state; aceita CPF (11) e CNPJ (14) |
| PF/PJ — site/ | ✅ implementado | Toggle PF/PJ no cadastro; person_type lido do JWT |
| PF/PJ — limites/taxas | 🔴 ausente | PJ e PF têm mesmas condições no Gatebox |
| Importador Universal de Produtos | 🔴 não existe | Criar crate apicash-importer |
| Ranking/Reputação/Selos | 🔴 não existe | Score antifraude existe, sistema separado não |
| Logística (Correios/Jadlog) | 🔴 não existe | Nenhuma integração |
| Disputas Gatebox | 🔴 0% | Tabela, endpoints, telas |
| Testes | 🔴 2% | Zero cobertura real |

---

## Fase 0 — Fundação [x]
**Duração: 1-2 dias | Prioridade: AGORA**

### 0.1 Ativar Postgres em produção [x]
Sem isso todos os dados somem ao reiniciar. Editar `money/.env`:
```env
APICASH_ORDERS_PG=1
APICASH_CUSTODY_PG=1
APICASH_SCORES_PG=1
DATABASE_URL=postgresql://...
```
Rodar: `cd money && ./runinfra.sh migrate`
(6 migrations SQL já existem em `money/apicash/migrations/`)

### 0.2 Commit do estado atual [x]
- `.gitignore` — corrigido com `**/.vite/`
- `ajustes.txt` — atualizado com stack real, importador universal, front-template

### 0.3 Atualizar money/.env.example [x]
Incluir todas as variáveis novas (Soroban, WhatsApp, PG flags).

**Verificação:** `./runinfra.sh status` → Postgres UP; `cargo run -p apicash-cli -- test-flow` → pedido persistido após restart.

---

## Fase 7.1 — Corrigir bug PJ/CNPJ no Gateway Next [x]
**Duração: 1 dia | Prioridade: AGORA (bug crítico)**

**Arquivo:** `gatebox/gatebox-rust/src/core/gateways/services/next.rs`, linha ~108

Substituir:
```rust
person_type: "NATURAL_PERSON".to_string()
```
Por:
```rust
person_type: match account.type_person_id {
    2 => "LEGAL_PERSON",
    _ => "NATURAL_PERSON",
}.to_string()
```
Isso bloqueia PIX de CNPJ de ir como NATURAL_PERSON para o banco parceiro.

---

## Fase 1 — Integrar site/ com o backend [x]
**Duração: 1 semana | Sprint 1**
**Objetivo:** Transformar `site/` de mock para produto real.

### 1.1 Criar API client [x]
`site/src/lib/api-client.ts` criado com:
- Proxy Vite dev: `/auth`, `/orders`, `/proposals`, `/custody` → localhost:3000
- tokenStore em localStorage (access + refresh)
- Interceptor de refresh automático (singleton Promise anti-race)
- Todos os endpoints: login, orders, proposals, custody/release

### 1.2 Auth real [x]
- `POST /auth/login` → `AppLogin.tsx` chama `login()` do contexto
- Access + refresh em localStorage via `tokenStore`
- `UserRoleContext` estendido: user identity, isAuthenticated, login(), logout()
- `RequireAuth` guard: redireciona `/login` se não autenticado
- Rotas `/buyer/*` e `/seller/*` protegidas no router

### 1.3 Fluxo Buyer — 5 telas [x]
1. `AppOrders` → [x] `GET /orders?role=buyer` via TanStack Query; lista real com status chips
2. `AppOrders/:id` → [x] `GET /orders/{id}` via TanStack Query + loading/error states
3. `AppPayment` → [x] aceita `{ pixBrCode, amount, orderId }` via route state; copy real
4. `AppWallet` → [x] `GET /wallet` — saldo disponível + em custódia em tempo real
5. `TransactionComplete` → [x] recebe `{ orderId, amount }` via route state
- Confirm delivery → [x] `POST /custody/release` com mutation + loading
- Open dispute → [x] `POST /orders/{id}/dispute` com mutation + loading

### 1.4 Fluxo Seller — 4 telas [x]
1. `SellerDashboard` → [x] `GET /seller/dashboard` — KPIs reais (volume, completados, custódia) + `GET /orders?role=seller`
2. `SellerOrders` → [x] botão "Nova Proposta" → `POST /proposals`; exibe ID gerado para compartilhar
3. `SellerDisputes` → [x] filtra orders com status disputed; empty state honesto quando sem disputas
4. `SellerWallet` → [x] `GET /wallet` + `GET /orders?role=seller` — saldo + histórico real

### 1.5 UI do Importador (placeholder) [x]
Botão "Importar Produto" em SellerOrders → dialog com campo URL + mensagem "em breve".
O endpoint `POST /v1/listings/import` será criado na Fase 3.1.

**Verificação:** Login real → criar proposta → buyer aceita → PIX code → confirmar entrega.

---

## Fase 7.2–7.5 — PF/PJ: Auth, UI e WhatsApp [~]
**Duração: 3-4 dias | Sprint 1 (paralela com Fase 1)**

### 7.2 Propagar person_type no JWT [x]
- `PersonType` enum (natural/legal) em `apicash-auth/models/claims.rs`
- `from_document()` infere pelo comprimento: 11=CPF → Natural, 14=CNPJ → Legal
- `UserIdentity` com campos `person_type` e `document`
- `APICASH_AUTH_USERS` aceita 4o campo: `user:pass:role:documento`
- `generate_token_full()` + `generate_refresh_token()` propagam os campos
- Rotação de refresh preserva person_type + document no ciclo completo
- `PersonType` re-exportado em `apicash-auth::lib.rs`

### 7.3 Score PJ diferenciado no antifraude [x]
**Arquivos:** `apicash-antifraude/src/score/risk_factors.rs`, `score_calculator.rs`, `behavioral_context.rs`, `antifraude_service.rs`
- `build_score()` recebe `doc_type: DocumentType` como novo parâmetro
- CPF: mantém pesos originais (Valid=+350, Invalid=-320)
- CNPJ: pesos diferenciados (Valid/ATIVA=+100, Invalid/BAIXADA=-200)
- `BehavioralContext` ganha `company_age_months: Option<u32>` (populado via SEFAZ futuramente)
- Empresa >24 meses → +50; <6 meses → -150
- `RiskFactor` ganha variantes `CnpjStatus` e `CompanyAge`
- `antifraude_service.rs` infere `DocumentType` por tamanho do documento (14 dígitos = CNPJ)
- 3 novos testes: CNPJ ativa+antiga, CNPJ ativa+nova, CNPJ inativa — todos passam

### 7.4 Fluxo WhatsApp para CNPJ [x]
- `WA_ESCROW_PLACEHOLDER_CPF` removido (era risco de segurança em produção)
- Novo estado `AwaitingBuyerDocument` na máquina de estados conversacional
- Após buyer aceitar proposta → bot pede CPF/CNPJ
- `parse_document()` aceita 11 (CPF) ou 14 (CNPJ) dígitos
- `finalize_order_after_buyer_accepted()` recebe `document: &str` real
- Templates `ask_buyer_document()` e `invalid_document()` adicionados

### 7.5 Cadastro PF/PJ no site/ [x]
**Arquivos:** `site/src/pages/app/AppLogin.tsx`, `site/src/contexts/UserRoleContext.tsx`
- Toggle PF/PJ no dialog de cadastro: Pessoa Física (CPF, ícone User) / Pessoa Jurídica (CNPJ, ícone Building2)
- Campo nome muda dinamicamente: "Nome completo" (PF) / "Razão social" (PJ)
- Placeholder CPF/CNPJ muda conforme seleção
- `buildIdentity()` agora lê `person_type` do JWT claim → `"legal"` vira `"pj"`, qualquer outro → `"pf"`
- `personType` propagado do servidor ao contexto em vez de hardcodado como `"pf"`

---

## Fase 2 — Admin Holdfy com front-template/ [x]
**Duração: 3-4 dias | Sprint 2**

### 2.1 Criar holdfy-admin/ [x]
Criado como app Vite+React+MUI standalone em `holdfy-admin/` (porta 3010).
Não duplica os 500+ arquivos do template — usa MUI diretamente.
Proxy Vite: `/admin/*` → `localhost:3001` (apicash-admin-backend).
Autenticação: API Key via localStorage → header `X-API-Key`.

Telas implementadas:
- **Login** — tela de entrada com API Key (valida chamando `/admin/dashboard`)
- **Dashboard** — 4 KPIs: Volume Total, Yield Acumulado, Disputas Abertas, Custódias Travadas
- **Pedidos** — DataGrid com filtro por status; colunas: ID, Valor, Status, Score, Decisão, Data
- **Disputas** — DataGrid com botão "Resolver" → dialog de resolução (FavorBuyer/FavorSeller/Split/Rejected)
- **Usuários/Score** — DataGrid com slider de score máximo; exibe score e nível de risco por cor
- **Yield Report** — 3 KPIs (yield total, custódias, liberadas) com filtro de período por data

### 2.2 apicash-admin-backend handlers [x]
Já completos desde a sessão anterior. Endpoints corretos confirmados:
- `GET /admin/dashboard` ✅
- `GET /admin/orders` ✅
- `GET /admin/disputes` + `POST /admin/disputes/{id}/resolve` ✅
- `GET /admin/users/score` ✅
- `GET /admin/reports/yield` ✅

Auth: `X-API-Key` = `APICASH_ADMIN_API_KEY` env var.

---

## Fase 5 — Segurança e Auditoria [~]
**Duração: 1 semana | Sprint 2 (paralela com Fase 2)**

### 5.1 Itens críticos Gatebox [~]
- [x] Senhas em plaintext → bcrypt/argon2: `verify_password()` com fallback transparente; `change_password` grava bcrypt hash; clientes e admin atualizados
- [x] JWT sem refresh token → `create_refresh_token()` + `rotate_refresh_token()` em `modules/shared/auth.rs`; endpoint `/auth/refresh` em admin handler; `token_type: "access"|"refresh"` nos claims
- [x] CORS não restrito → `CorsLayer` via `tower-http` configurado por `GATEBOX_CORS_ORIGINS` (vazio/"*"=any, lista=whitelist); adicionado ao app em `server.rs`
- [x] Rate limiting → `LoginRateLimiter` com sliding window 5min/5 tentativas + lockout 15min; HTTP 429 com `Retry-After`
- [x] Audit log → `AppLogRepository.insert()` adicionado; admin login success/failure e token refresh gravados

### 5.2 Itens críticos APICash [x]
- [x] `APICASH_AUTH_DISABLED=true` → `AuthConfig::validate_startup_safety()` chamado em `main.rs`; requer `APICASH_INSECURE_DEV=1` para override em dev
- [x] HMAC webhook `POST /internal/webhook/pix` → já implementado com `verify_hmac()` + `GATEBOX_WEBHOOK_SECRET`; aceita sem verificação só se secret ausente (dev)
- [x] `APICASH_JWT_SECRET` com entropia suficiente → validado no startup (mínimo 32 chars)
- [x] Logs com dados PIX/CPF → `cached_document_validator` e `http_document_validator` mascarados (doc_prefix[3] e doc_type, sem número completo)

### 5.3 Testes mínimos [~]
- [x] `apicash-antifraude` → 39 testes passando (ScoreCalculator CPF+CNPJ, validadores, antifraude comportamental)
- [x] `apicash-custody` → 9 testes passando: lock/release/yield split + 6 testes unitários YieldCalculator (zero days, negativo, precisão, zero principal, taxa customizada)
- [x] `apicash-auth` → 4 testes de segurança: validate_startup_safety (auth_disabled, jwt_secret curto, valid config)
- [ ] `apicash-core` → fluxo order → settle → release (integração) — pendente
- [x] `gatebox-rust` → cálculo saldo = CREDIT - DEBIT - MED: helpers `compute_available_balance`, `calculate_med_amount` + 13 testes em `service_async.rs`

Framework: `cargo test` + `axum::test`.

---

## Fase 7.6–7.7 — Limites PJ e Admin PF/PJ [x]
**Duração: 2-3 dias | Sprint 2**

### 7.6 Limites/taxas diferenciadas no Gatebox [x]
- [x] Migration `20260524000000_fees_person_type.sql`: coluna `person_type_id` na tabela `fees`
- [x] `fees/ddl.rs`: `SQL_GET_BY_PERSON_TYPE`; `fees/repository.rs`: `get_by_person_type()` trait + impl
- [x] `provider_selector.rs`: fallback layered account-fee → person-type fee → no-fee
- [x] `webhook_service.rs`: aviso soft-limit PIX IN por tipo (PF ≤ R$20k, PJ ≤ R$500k) — warn, não block

### 7.7 Admin com filtro PF/PJ [x]
- [x] front-gatebox `Customers`: ToggleButtonGroup PF/PJ, coluna Tipo+Documento com Chip
- [x] holdfy-admin `Scores`: ToggleButtonGroup PF/PJ, coluna person_type com Chip
- [x] holdfy-admin `Orders`: ToggleButtonGroup PF/PJ, coluna person_type

---

## Fase 3.2 — Disputas no Gatebox [x]
**Duração: 3-4 dias | Sprint 3**

**Arquivos a criar:** `gatebox/gatebox-rust/src/modules/disputes/`

Migration SQL nova:
```sql
CREATE TABLE disputes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  transaction_id UUID REFERENCES transaction(id),
  account_id BIGINT NOT NULL,
  type VARCHAR(20) NOT NULL,   -- INFRACTION | REVERSAL | FRAUD
  status VARCHAR(20) DEFAULT 'OPEN',
  reason TEXT,
  evidence JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  resolved_at TIMESTAMPTZ,
  resolved_by BIGINT
);
```

Endpoints:
- `POST /admin/disputes` — abrir
- `GET /admin/disputes` — listar (filtros: status, tipo, período, PF/PJ)
- `PUT /admin/disputes/:id/resolve`
- `GET /customers/disputes`
- Webhook: receber infração de SevenTrust/Sulcred

Tela: criar `gatebox/front-gatebox/src/layouts/admin/disputes/` (seguir padrão das outras telas admin).

---

## Fase 3.4 — WhatsApp Multi-device Bridge [x]
**Duração: 2-3 dias | Sprint 3**

**Arquivo:** `money/apicash/crates/apicash-whatsapp/src/wa_multidevice.rs` (ver comentário linhas 4-14)

O que falta:
- Consumir `whatsapp_rust::Event` do loop de eventos
- Mapear `whatsapp_rust::Event::Message` → `WhatsAppEvent`
- Fechar ciclo: QR pairing → autenticado → receber msgs → despachar ao `MessageHandler`

Padrão a seguir: webhook Cloud API em `whatsapp_service.rs` linhas 71-79 — replicar a mesma lógica para o transporte multi-device.

---

## Fase 6 — Soroban Testnet Live [ ]
**Duração: 2-3 dias | Sprint 4**

```bash
cd money/apicash
scripts/bootstrap-testnet-env.sh    # funda identidades holdfy-*
scripts/soroban-testnet-deploy.sh   # deploy contrato → imprime CONTRACT_ID
```

Atualizar `money/.env`:
```env
APICASH_SOROBAN_ENABLED=1
APICASH_SOROBAN_STRICT=1
APICASH_SOROBAN_ESCROW_CONTRACT_ID=<id>
APICASH_FIAT_RAIL=anchor
```

Compilar: `cargo build -p apicash-custody --features soroban`
Verificar: `scripts/testnet-onchain-smoke.sh` → hash real no Stellar Explorer.

---

## Fase 3.1 — Importador Universal de Produtos [x]
**Duração: 1 semana | Sprint 4**
**Novo crate:** `money/apicash/crates/apicash-importer/`

Implementado:
- `JsonLdExtractor` — `schema.org/Product` JSON-LD (cobre OLX, Shopee, maioria dos e-commerces)
- `OpenGraphExtractor` — `og:title`, `og:image`, `og:description` (Instagram, Facebook, TikTok)
- `MercadoLivreExtractor` — API oficial `api.mercadolibre.com/items/{id}` com detecção de MLB{id}
- `LlmExtractor` — fallback: envia HTML ao Claude API (claude-haiku-4-5), requer `ANTHROPIC_API_KEY`
- `ImporterService` — cadeia de extratores, `User-Agent` próprio, timeout 15s
- Endpoint `POST /v1/listings/import` em `apicash-core/src/handlers/importer_handler.rs`
- `AppState.importer: Arc<ImporterService>` injetado
- Proxy Vite `/v1` → localhost:3000
- `api.importListing(url)` em `api-client.ts`
- Dialog em `SellerOrders.tsx`: importa, exibe preview (foto + título + preço), botão "Criar Proposta"

Pendente (MVP2):
- Cache Redis por URL (TTL 5min)
- Fila Pulsar para async
- `ImageDownloader` para re-hospedar fotos
- Integração WhatsApp (detectar URL enviada → importar → preview)

---

## Fase 3.3 — Ranking, Reputação e Selos [x]
**Implementado:**
- `apicash-antifraude/src/reputation/`: `UserReputation`, `ReputationSeal` (Verified/Premium/Authenticated), `ReputationService`
- `GET /reputation/{user_id}` com auth (self ou admin)
- `AppState.reputation: Arc<ReputationService>` injetado
- `ReputationBadge.tsx` no `site/`: badge colorido com ícone, tooltip com score e txns
- Perfil comprador (`AppProfile`) e vendedor (`SellerProfile`) exibem badge
- `api.getReputation()` + tipos `ReputationSeal`/`ReputationResponse` em `api-client.ts`
- 8 testes em `reputation_model.rs` (no_kyc, too_few_txns, verified, premium, authenticated)

**Arquivos chave:**
- `crates/apicash-antifraude/src/reputation/`
- `crates/apicash-core/src/handlers/reputation_handler.rs`
- `site/src/components/ReputationBadge.tsx`

**Notas de design:**


```rust
struct UserReputation {
    user_id: Uuid,
    role: UserRole,
    score: i32,                       // 0-1000
    seal: Option<Seal>,               // None | Verified | Premium | Authenticated
    completed_transactions: u32,
    dispute_rate: Decimal,
    on_time_delivery_rate: Decimal,
    last_updated: DateTime<Utc>,
}
```

Lógica de selos:
- `Verified`: KYC aprovado + documento válido + ≥5 transações sem disputa
- `Premium`: Verified + score ≥800 + ≥20 transações
- `Authenticated`: Premium + ≥50 transações + taxa disputa <2%

Expor como badges no `site/` nas telas de perfil e pedidos.

---

## Fase 4 — Logística [x]
**Duração: 1 semana | Sprint 5**
**Novo crate:** `money/apicash/crates/apicash-logistics/`

Implementado:
- `MelhorEnvioClient` — HTTP client com Bearer token + User-Agent correto; sandbox/prod via `MELHOR_ENVIO_SANDBOX`
- `LogisticsService` com `quote()`, `generate_label()`, `track()`
- Tipos: `ShippingQuote`, `ShippingLabel`, `TrackingInfo`, `TrackingEvent`, `CarrierCode` (PAC/SEDEX/Jadlog)
- Endpoints em `apicash-core`:
  - `POST /logistics/shipping/quote`
  - `POST /logistics/shipping/label`
  - `GET /logistics/tracking/{code}`
- `AppState.logistics: Arc<LogisticsService>` — fallback graceful se token ausente
- Proxy Vite `/logistics` → localhost:3000
- `api.quoteShipping()` + `api.trackShipment()` em `api-client.ts`
- Variáveis: `MELHOR_ENVIO_TOKEN` (obrigatória), `MELHOR_ENVIO_SANDBOX=1` (dev)

Pendente (MVP2):
- Tela de cotação no `site/` (estimativa de frete no checkout)
- Integração WhatsApp: "rastrear {código}" → `track()` → resposta formatada

---

## Ordem de execução

```
Fase 0   (Postgres)             → 1-2 dias   — AGORA
Fase 7.1 (bug gateway CNPJ)     → 1 dia      — AGORA
Fase 1   (site/ integrado)      → 1 semana   — Sprint 1
Fase 7.2-7.5 (PF/PJ auth+UI)   → 3-4 dias   — Sprint 1 (paralelo)
Fase 2   (admin holdfy)         → 3-4 dias   — Sprint 2
Fase 5   (segurança/testes)     → 1 semana   — Sprint 2 (paralelo)
Fase 7.6-7.7 (limites PJ)      → 2-3 dias   — Sprint 2 (paralelo)
Fase 3.2 (disputas Gatebox)    → 3-4 dias   — Sprint 3
Fase 3.4 (WhatsApp bridge)     → 2-3 dias   — Sprint 3
Fase 6   (Soroban live)         → 2-3 dias   — Sprint 4
Fase 3.1 (importador universal) → 1 semana   — Sprint 4
Fase 3.3 (ranking/selos)       → 3-4 dias   — Sprint 5
Fase 4   (logística)            → 1 semana   — Sprint 5
```

**Total estimado: 7-9 semanas**

---

## Arquivos críticos por fase

| Fase | Arquivos |
|---|---|
| 0 | `money/.env`, `money/apicash/migrations/` |
| 7.1 | `gatebox/gatebox-rust/src/core/gateways/services/next.rs` linha ~108 |
| 1 | `site/src/lib/api-client.ts` (criar), `site/src/pages/app/*`, `site/src/pages/seller/*` |
| 7.2 | `money/apicash/crates/apicash-auth/src/models/claims.rs`, `apicash-shared/src/domain/` |
| 7.3 | `money/apicash/crates/apicash-antifraude/src/score/risk_factors.rs` |
| 7.4 | `money/apicash/crates/apicash-whatsapp/src/handlers/order_flow.rs` |
| 7.5 | `site/src/pages/app/AppLogin.tsx`, `site/src/contexts/` |
| 2 | `holdfy-admin/` (criar de front-template/), `apicash-admin-backend/src/handlers/` |
| 3.1 | `money/apicash/crates/apicash-importer/` (criar), `apicash-core/src/router.rs` |
| 3.2 | `gatebox/gatebox-rust/src/modules/disputes/` (criar), nova migration |
| 3.3 | `money/apicash/crates/apicash-antifraude/src/` |
| 3.4 | `money/apicash/crates/apicash-whatsapp/src/wa_multidevice.rs` linhas 4-14 |
| 4 | `money/apicash/crates/apicash-logistics/` (criar) |
| 5 | `gatebox/gatebox-rust/src/modules/*/handler.rs`, `cargo test --workspace` |
| 6 | `money/apicash/scripts/soroban-testnet-deploy.sh`, `money/.env` |

---

## Decisões arquiteturais (não mudar sem revisão)

1. **`site/` é o produto Holdfy** — Vite + shadcn/ui + Tailwind. Não migrar para MUI.
2. **`front-template/` é apenas para admin** — Material Dashboard 3 PRO para backoffice/admin.
3. **`apicash-importer` crate separado** — injetar via trait, não misturar com apicash-core.
4. **Postgres obrigatório em produção** — in-memory só para testes unitários.
5. **Soroban mock aceitável no MVP** — alternância por env var, não por compile-time feature.
6. **Redis para importer e sessões WhatsApp** — ativar nessas duas áreas primeiro.
7. **Wallet Go = descartar** — saldo vem do APICash (escrow) + Gatebox (PIX).
8. **PF e PJ são cidadãos de primeira classe** — propagar `person_type` em todo o sistema.
