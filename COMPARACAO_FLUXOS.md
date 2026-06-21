# WhatsApp vs Web — Comparação de Funcionalidades

> Legenda: ✅ implementado · ❌ ausente · ⚠️ parcial

---

## Identidade e Autenticação

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| Autenticação | Peer key (número WA) | JWT HS256 (access + refresh) |
| Cadastro PF | CPF enviado no chat | Toggle + campo CPF com máscara |
| Cadastro PJ | CNPJ enviado no chat | Toggle + campo CNPJ com máscara |
| Máscara de telefone | — | ✅ `maskPhone` nos perfis |
| Consulta Receita Federal (KYC) | ❌ só validação matemática | ✅ on-blur no cadastro e no pagamento |
| Score antifraude CPF/CNPJ | ✅ diferenciado (11 fatores) | ✅ diferenciado (11 fatores) |
| Reputação / Selos | ❌ | ✅ badge Verified / Premium / Authenticated |

---

## Importação de Produtos

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| Detecção automática de URL | ✅ no chat | ❌ dialog manual |
| Mercado Livre (API oficial) | ✅ | ✅ |
| OLX / Shopee (JSON-LD) | ✅ | ✅ |
| Instagram / Facebook (OpenGraph) | ✅ | ✅ |
| TikTok | ✅ | ✅ |
| E-commerce genérico | ✅ | ✅ |
| Fallback LLM (gpt-4o-mini) | ✅ | ✅ |
| Fotos re-hospedadas no MinIO | ✅ | ✅ |
| Listing salvo no Postgres | ✅ | ✅ |
| Cache Redis por URL (5 min) | ✅ | ✅ |
| Fila async Pulsar / NATS | ❌ | ✅ `/v1/listings/import/async` |
| Auditoria MongoDB | ✅ `wa_messages` | ✅ `web_listing_imports` |
| Preview da foto no chat | ✅ envia imagem bytes | ❌ exibe URL |
| Pré-preenchimento de proposta | ✅ automático | ✅ preenche form |

---

## Fluxo de Compra e Venda

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| Criação de proposta | ✅ conversacional (bot guia) | ✅ dialog manual |
| Proposta aberta (link) | ✅ | ✅ (buyer_id omitido) |
| Aceite de proposta | ✅ mensagem de confirmação | ✅ tela dedicada (`AppPayment`) |
| PIX QR Code | ✅ enviado como imagem | ✅ gerado via `qrcode` |
| Polling de pagamento | ✅ webhook PIX → atualiza estado | ✅ auto-poll a cada 5s |
| Confirmação de entrega | ✅ comprador envia texto | ✅ botão com AlertDialog |
| Off-ramp PIX automático | ✅ após release Soroban | ✅ `POST /orders/:id/off-ramp` |
| Chave PIX vendedor | ✅ solicitada no chat | ✅ campo na proposta + perfil |
| Histórico de pedidos | ❌ estado de sessão apenas | ✅ listagem com filtros |
| Saldo / wallet | ❌ | ✅ `AppWallet` |
| Cotação de frete | ❌ | ✅ seção colapsável no checkout |

---

## Rastreio de Encomenda

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| Registro do código pelo vendedor | ✅ envia no chat | ✅ campo no detalhe do pedido |
| Persistência em `order_tracking_status` | ✅ | ✅ |
| `buyer_peer` preenchido | ✅ (peer WA da sessão) | ✅ buscado de `wa_contacts` |
| `seller_peer` preenchido | ✅ (peer WA da sessão) | ✅ buscado de `wa_contacts` |
| Notificação WA ao comprador | ✅ proativo (poll 30 min) | ✅ via tracking monitor (shared) |
| Notificação WA ao vendedor | ✅ proativo | ✅ via tracking monitor (shared) |
| Consulta sob demanda | ❌ | ✅ botão "Atualizar" + `TrackingCard` |
| Providers: Correios → LinkTrack → Melhor Envio | ✅ circuit breaker | ✅ mesma `LogisticsService` |
| Captura de telefone do vendedor | via sessão WA | ✅ campo na proposta → `wa_contacts` |
| Captura de telefone do comprador | via sessão WA | ✅ campo no aceite → `wa_contacts` |

---

## Disputas

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| Abertura de disputa | ✅ mensagem no chat | ✅ dialog com nota |
| Upload de evidências (fotos) | ✅ envia imagem bytes | ✅ até 5 arquivos base64 |
| Upload de vídeo | ✅ | ✅ |
| Auto-resolução via LLM | ✅ OpenAI analisa conversa | ⚠️ endpoint existe, UI não expõe ainda |
| Vendedor contesta via WA | ✅ | ❌ |
| Notificação do resultado | ✅ WA a ambas as partes | ❌ |
| Auditoria MongoDB | ✅ `wa_conversation_summaries` | ✅ `web_order_events` |

---

## Blockchain / Custódia

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| Custódia em escrow | ✅ Soroban ou mock | ✅ Soroban ou mock |
| BRLx on-ramp (Anchor) | ✅ disparado ao pagar | ✅ disparado ao pagar |
| Lock Soroban | ✅ em background | ✅ em background |
| Release Soroban | ✅ ao confirmar entrega | ✅ ao confirmar entrega |
| x402 micropayments | ❌ | ✅ (opcional, `APICASH_X402_REQUIRED`) |

---

## Auditoria e Observabilidade

| Funcionalidade | WhatsApp | Web (site/) |
|---|:---:|:---:|
| MongoDB — mensagens | ✅ `wa_messages` (cada msg) | — |
| MongoDB — importações | — | ✅ `web_listing_imports` |
| MongoDB — eventos de pedido | ✅ `wa_conversation_summaries` | ✅ `web_order_events` |
| Logs estruturados (tracing) | ✅ | ✅ |
| Audit log de segurança | ✅ | ✅ |

---

## Gaps prioritários (ordem de trabalho)

| # | O que falta | Canal | Complexidade |
|---|---|---|---|
| 1 | Consulta Receita Federal (KYC) no chat | WhatsApp | baixa |
| 2 | Reputação / Selos no chat | WhatsApp | baixa |
| 3 | Detecção automática de URL no web (sem dialog) | Web | média |
| 4 | Fila async Pulsar/NATS no WhatsApp | WhatsApp | média |
| 5 | Preview de foto enviada como bytes no web | Web | baixa |
| 6 | Histórico de pedidos no WhatsApp | WhatsApp | alta |
| 7 | Saldo / wallet no WhatsApp | WhatsApp | média |
| 8 | Cotação de frete no WhatsApp | WhatsApp | baixa |
| 9 | Consulta de rastreio sob demanda no WhatsApp | WhatsApp | baixa |
| 10 | Auto-resolução LLM exposta na UI web | Web | baixa |
| 11 | Vendedor contesta disputa pelo site | Web | média |
| 12 | Notificação WA do resultado da disputa (web) | Web | baixa |
