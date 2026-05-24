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
| PF/PJ — antifraude valida CPF e CNPJ | ✅ completo | Score idêntico para ambos (ok por ora) |
| PF/PJ — Gatebox modelos (type_person_types) | ✅ presente | Seeds: NATURAL_PERSON(1), LEGAL_PERSON(2) |
| PF/PJ — Gateway Next | 🔴 BUG CRÍTICO | Hardcoda NATURAL_PERSON para CNPJ também |
| PF/PJ — Auth/JWT claims | 🔴 ausente | `person_type` não propagado nos tokens |
| PF/PJ — WhatsApp | 🔴 ausente | Só coleta CPF placeholder hardcoded |
| PF/PJ — site/ | 🔴 ausente | Sem campos CNPJ no cadastro/login |
| PF/PJ — limites/taxas | 🔴 ausente | PJ e PF têm mesmas condições no Gatebox |
| Importador Universal de Produtos | 🔴 não existe | Criar crate apicash-importer |
| Ranking/Reputação/Selos | 🔴 não existe | Score antifraude existe, sistema separado não |
| Logística (Correios/Jadlog) | 🔴 não existe | Nenhuma integração |
| Disputas Gatebox | 🔴 0% | Tabela, endpoints, telas |
| Testes | 🔴 2% | Zero cobertura real |

---

## Fase 0 — Fundação [ ]
**Duração: 1-2 dias | Prioridade: AGORA**

### 0.1 Ativar Postgres em produção [ ]
Sem isso todos os dados somem ao reiniciar. Editar `money/.env`:
```env
APICASH_ORDERS_PG=1
APICASH_CUSTODY_PG=1
APICASH_SCORES_PG=1
DATABASE_URL=postgresql://...
```
Rodar: `cd money && ./runinfra.sh migrate`
(6 migrations SQL já existem em `money/apicash/migrations/`)

### 0.2 Commit do estado atual [ ]
- `.gitignore` — corrigido com `**/.vite/`
- `ajustes.txt` — atualizado com stack real, importador universal, front-template

### 0.3 Atualizar money/.env.example [ ]
Incluir todas as variáveis novas (Soroban, WhatsApp, PG flags).

**Verificação:** `./runinfra.sh status` → Postgres UP; `cargo run -p apicash-cli -- test-flow` → pedido persistido após restart.

---

## Fase 7.1 — Corrigir bug PJ/CNPJ no Gateway Next [ ]
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

## Fase 1 — Integrar site/ com o backend [ ]
**Duração: 1 semana | Sprint 1**
**Objetivo:** Transformar `site/` de mock para produto real.

### 1.1 Criar API client [ ]
Criar `site/src/lib/api-client.ts` com TanStack Query (já instalado).
- Base URL: APICash core porta 3000, admin porta 3001
- Auth: JWT em localStorage com interceptor de refresh automático

### 1.2 Auth real [ ]
- `POST /auth/login` → substituir mock em `site/src/pages/app/AppLogin`
- Armazenar access + refresh token
- Proteger rotas `/buyer/*` e `/seller/*` (UserRoleContext já existe)

### 1.3 Fluxo Buyer — 5 telas [ ]
1. `AppOrders` → `GET /orders`
2. `AppOrders/:id` → `GET /orders/{id}`
3. `AppPayment` → `POST /orders` + `POST /payments/pix` → retorna PIX code
4. `AppWallet` → saldo calculado dos pedidos
5. `TransactionComplete` → `POST /custody/release`

### 1.4 Fluxo Seller — 4 telas [ ]
1. `SellerDashboard` → `GET /dashboard` (apicash-admin-backend)
2. `SellerOrders` → `GET /proposals` + `GET /orders`
3. `SellerDisputes` → `GET /orders?status=Disputed`
4. `SellerWallet` → saldo + yield acumulado

### 1.5 UI do Importador (placeholder) [ ]
Campo "Cole o link do produto" em SellerOrders — apenas UI + loading state.
O endpoint `POST /v1/listings/import` será criado na Fase 3.1.

**Verificação:** Login real → criar pedido → ver PIX code → confirmar entrega.

---

## Fase 7.2–7.5 — PF/PJ: Auth, UI e WhatsApp [ ]
**Duração: 3-4 dias | Sprint 1 (paralela com Fase 1)**

### 7.2 Propagar person_type no JWT [ ]
**Arquivos:** `apicash-auth/src/models/claims.rs`, `apicash-shared/src/domain/`
```rust
pub enum PersonType { Natural, Legal }
pub struct UserIdentity {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub person_type: PersonType,  // NOVO
    pub document: String,         // CPF ou CNPJ
}
```

### 7.3 Score PJ diferenciado no antifraude [ ]
**Arquivo:** `apicash-antifraude/src/score/risk_factors.rs`
Adicionar para `DocumentType::Cnpj`:
- `+100` CNPJ ATIVA na Receita Federal
- `-200` CNPJ BAIXADA ou SUSPENSA
- `+50` empresa com >2 anos
- `-150` empresa com <6 meses

### 7.4 Fluxo WhatsApp para CNPJ [ ]
**Arquivo:** `apicash-whatsapp/src/handlers/order_flow.rs`
```
Bot: "Você é pessoa física (CPF) ou jurídica (CNPJ)?"
→ PF: validar 11 dígitos
→ PJ: validar 14 dígitos + pedir razão social
```
Remover `WA_ESCROW_PLACEHOLDER_CPF = "52998224725"` (risco de segurança).

### 7.5 Cadastro PF/PJ no site/ [ ]
**Arquivo:** `site/src/pages/app/AppLogin.tsx` e telas de perfil
- Toggle PF/PJ no formulário
- PF: CPF + nome + data nascimento
- PJ: CNPJ + razão social + nome fantasia + CPF do responsável
- Adicionar `personType: 'pf' | 'pj'` ao `UserRoleContext`

---

## Fase 2 — Admin Holdfy com front-template/ [ ]
**Duração: 3-4 dias | Sprint 2**

### 2.1 Criar holdfy-admin/ a partir de front-template/ [ ]
Clonar `front-template/` para `holdfy-admin/` e customizar.
Mapear telas para layouts prontos do Material Dashboard 3 PRO:
- Dashboard → `layouts/dashboards/analytics/` (KPIs: pedidos, volume, yield, fraudes bloqueadas)
- Pedidos → `layouts/ecommerce/orders/`
- Disputas → criar custom (listar, julgar, resolver)
- Usuários/Scores → `layouts/pages/users/`
- Yield Reports → `layouts/dashboards/sales/`
- Auth → `layouts/authentication/sign-in/` (pronto)

### 2.2 Completar apicash-admin-backend handlers [ ]
**Arquivo:** `money/apicash/crates/apicash-admin-backend/src/handlers/`
- `GET /dashboard` → KPIs reais do Postgres
- `GET /orders` → paginado, com filtros
- `GET /disputes` → disputas abertas
- `GET /scores` → usuários por faixa de risco (APPROVE/REVIEW/BLOCK)
- `GET /yield-reports` → yield distribuído por período

Auth: `X-API-Key` (já implementado).

**Verificação:** Admin dashboard mostrando KPIs reais do Postgres.

---

## Fase 5 — Segurança e Auditoria [ ]
**Duração: 1 semana | Sprint 2 (paralela com Fase 2)**

### 5.1 Itens críticos Gatebox [ ]
- [ ] Senhas em plaintext → bcrypt/argon2
- [ ] JWT sem refresh token → adicionar
- [ ] CORS não restrito → whitelist em produção
- [ ] Rate limiting ausente → adicionar nos endpoints críticos
- [ ] Audit log modelado mas não usado → ativar nos handlers

### 5.2 Itens críticos APICash [ ]
- [ ] `APICASH_AUTH_DISABLED=true` em produção → panic no startup se set
- [ ] HMAC webhook `POST /internal/webhook/pix` → validar que está sendo verificado
- [ ] `APICASH_JWT_SECRET` com entropia suficiente → validar no startup
- [ ] Logs com dados PIX/CPF → auditar calls `tracing::info!`

### 5.3 Testes mínimos [ ]
Prioridade: onde tem dinheiro envolvido.
1. `apicash-antifraude` → `ScoreCalculator` (11 fatores)
2. `apicash-custody` → `YieldCalculator` (36% APY)
3. `apicash-core` → fluxo order → settle → release (integração)
4. `gatebox-rust` → cálculo saldo = CREDIT - DEBIT - MED

Framework: `cargo test` + `axum::test`.

---

## Fase 7.6–7.7 — Limites PJ e Admin PF/PJ [ ]
**Duração: 2-3 dias | Sprint 2**

### 7.6 Limites/taxas diferenciadas no Gatebox [ ]
**Arquivos:** `gatebox-rust/regras_de_negocio.md` + migration nova na tabela `fees`
- Adicionar coluna `person_type_id` (FK → `type_person_types`)
- PJ: limite PIX maior; possibilidade de isenção por volume
- Impacto em `pix_principal/service.rs`: resolver taxa pelo `type_person_id` da conta

### 7.7 Admin com filtro PF/PJ [ ]
- front-gatebox: filtro PF/PJ na listagem de clientes
- holdfy-admin: KYC com campos de razão social + CNPJ + sócios para PJ
- Relatórios segmentados por tipo de pessoa

---

## Fase 3.2 — Disputas no Gatebox [ ]
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

## Fase 3.4 — WhatsApp Multi-device Bridge [ ]
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

## Fase 3.1 — Importador Universal de Produtos [ ]
**Duração: 1 semana | Sprint 4**
**Novo crate:** `money/apicash/crates/apicash-importer/`

Trait central:
```rust
trait Extractor: Send + Sync {
    async fn extract(&self, url: &Url) -> Result<ProductDraft, ImporterError>;
}
```

Pipeline em camadas (implementar nesta ordem):
1. `JsonLdExtractor` — `schema.org/Product` JSON-LD (cobre OLX, Shopee, maioria dos e-commerces)
2. `OpenGraphExtractor` — `og:title`, `og:image`, `og:description` (Instagram, Facebook, TikTok)
3. `MercadoLivreExtractor` — API oficial `api.mercadolibre.com/items/{id}`
4. `LlmExtractor` — fallback: envia HTML ao Claude API, extrai campos

Infraestrutura:
- `ImageDownloader` — baixa e re-hospeda fotos (nunca URL externa direta)
- `HeadlessFetcher` — Chromium via crate `chromiumoxide` (para JS dinâmico)
- Cache Redis TTL 5min por URL
- Fila Pulsar para processamento async
- Sandbox: validar URL, bloquear IPs internos, timeout

Endpoint: `POST /v1/listings/import` → body `{ url }` → response `ProductDraft { title, price_suggested, description, photos, source_url, source_platform }`

WhatsApp: em `message_handler.rs`, detectar URL enviada → acionar importer → preview → confirmar → criar proposta.

**Verificação:** `POST /v1/listings/import` com URL OLX real → retorna título e foto.

---

## Fase 3.3 — Ranking, Reputação e Selos [ ]
**Duração: 3-4 dias | Sprint 5**
**Adicionar em:** `money/apicash/crates/apicash-antifraude/src/` (ou novo crate `apicash-reputation`)

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

## Fase 4 — Logística [ ]
**Duração: 1 semana | Sprint 5**
**Novo crate:** `money/apicash/crates/apicash-logistics/`

MVP: integrar **Melhor Envio API** (agrega Correios + Jadlog, mais simples que Correios direto).
- `POST /logistics/shipping/quote` — cotação
- `POST /logistics/shipping/label` — gerar etiqueta (PDF)
- `GET /logistics/tracking/{code}` — rastreamento

WhatsApp: quando comprador digitar "rastrear" ou enviar código → chamar `apicash-logistics`.

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
