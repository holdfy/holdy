# LogisticaHoldFy

Simulador local de rastreio de encomendas para desenvolvimento HoldFy.

## Estrutura

```
apprastreio/
├── backend/   # API Rust (Axum) — porta 8092
└── app/       # Flutter LogisticaHoldFy (tema dark)
```

## Backend (Rust)

```bash
cd backend
cargo run
# ou: LOGISTICA_HTTP_PORT=8092 cargo run
```

### Endpoints

| Método | Rota | Descrição |
|--------|------|-----------|
| GET | `/health` | Health check |
| GET | `/presets` | Etapas pré-definidas de entrega |
| GET | `/trackers` | Lista rastreios simulados |
| POST | `/trackers` | Gera novo código (formato Correios) |
| GET | `/trackers/{code}` | Detalhe interno |
| POST | `/trackers/{code}/presets` | Adiciona etapa preset |
| GET | `/logistics/tracking/{code}` | Formato compatível com APICash |
| GET | `/tracking/{code}` | Alias de consulta pública |

Etapas preset disponíveis:

1. Centro de distribuição
2. Saiu rumo ao destino
3. Chegou ao destino
4. Saiu para entrega
5. Entregue

Dados em memória (reiniciam ao parar o servidor).

## App Flutter

```bash
cd app
flutter pub get
flutter run
```

Abas:

- **Rastreios** — criar códigos e simular etapas
- **Setup** — host/porta do backend (padrão `127.0.0.1:8092`)

Telefone físico na mesma Wi‑Fi: na aba Setup, informe o IP LAN do PC.

## Integração com APICash

Com `APICASH_TRACKING_MODE=simulated` em `money/.env`, o `apicash-logistics` consulta **apenas** este backend (sem Correios/LinkTrack/Melhor Envio).

```env
APICASH_TRACKING_MODE=simulated
APICASH_TRACKING_SIMULATOR_URL=http://127.0.0.1:8092
LOGISTICA_HTTP_PORT=8092
```

### Fluxo de teste

1. Subir o simulador: `cd backend && cargo run`
2. Subir APICash: `cd money && ./runapp.sh start apicash`
3. No app Flutter, criar um rastreio e adicionar etapas
4. Consultar o mesmo código via:
   - WhatsApp: `rastrear AA123456789BR`
   - Site: dialog "Rastrear Encomenda"
   - API: `GET http://localhost:3000/logistics/tracking/{code}`

O endpoint `GET /logistics/tracking/{code}` deste backend retorna JSON no formato `TrackingInfo` usado por `apicash-logistics`.
