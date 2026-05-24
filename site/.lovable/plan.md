

# APICash – Pagamento Protegido

Aplicação completa com **Landing Page institucional** e **Dashboard de gestão**, estilo moderno e minimalista inspirado em fintechs como Stripe e Nubank.

---

## Parte 1: Landing Page

### Hero Section
- Título: "Pagamento Protegido que Rende Enquanto Protege"
- Subtítulo explicativo sobre escrow + PIX + Stellar
- CTA principal: "Comece Agora" e "Ver Documentação"
- Ilustração/ícone representando proteção financeira

### Seção "Problema"
- Cards com os 4 problemas principais (golpes, chargebacks, falta de confiança)
- Ícones visuais para cada problema

### Seção "Como Funciona"
- Fluxo em 5 etapas com ícones numerados:
  1. Criação do Pedido → 2. Pagamento PIX → 3. Custódia Stellar → 4. Confirmação de Entrega → 5. Liberação do Pagamento
- Visual de timeline/stepper horizontal

### Seção "3 Pilares"
- Pagamento Protegido (escrow)
- On/Off-Ramp (PIX ↔ Stellar)
- Yield DeFi (opcional) – dinheiro rende em custódia

### Seção "Benefícios por Público"
- 3 colunas: Compradores | Vendedores | Plataformas
- Lista de benefícios para cada grupo

### Seção "Integração Técnica"
- API REST, Webhooks, SDKs
- Snippet de código exemplo
- Canais suportados (WhatsApp, OLX, Shopee, TikTok Shop, etc.)

### Seção "Modelo de Receita"
- Taxa por transação (1,5%–3%)
- Participação no yield DeFi
- Planos B2B

### Footer
- Links, contato, redes sociais

---

## Parte 2: Dashboard (área logada simulada)

### Sidebar de Navegação
- Visão Geral, Pedidos, Pagamentos, Disputas, Configurações

### Página Visão Geral
- Cards de métricas: Total em custódia, Pedidos ativos, Yield acumulado, Disputas abertas
- Gráfico de transações recentes (mock)

### Página Pedidos
- Tabela com pedidos protegidos (ID, comprador, vendedor, valor, status, data)
- Filtros por status: PENDING, IN_CUSTODY, COMPLETED, CANCELLED
- Botões de ação: Liberar, Cancelar, Abrir Disputa

### Página Pagamentos
- Lista de pagamentos com status (PENDING, IN_CUSTODY, RELEASED, REFUNDED)
- Detalhes do fluxo PIX → Stellar → PIX

### Página Disputas
- Lista de disputas abertas com status e resolução

### Página Configurações
- Configurações de webhook, chaves de API (mockadas)

---

## Design
- Paleta: tons de azul/verde (confiança e dinheiro), fundo branco, tipografia limpa
- Componentes shadcn/ui
- Responsivo para mobile e desktop
- Dados mockados para demonstração

