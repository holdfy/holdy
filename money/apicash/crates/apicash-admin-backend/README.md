# apicash-admin-backend

API HTTP **interna** para o dashboard administrativo (vendedores B2B, plataforma, equipe interna). O binário escuta **`0.0.0.0:3001`** por padrão, separado da API pública (`apicash-core`).

## Autenticação

Defina `APICASH_ADMIN_API_KEY`. Cada requisição deve enviar:

- `X-API-Key: <valor>`, ou  
- `Authorization: Bearer <valor>`

JWT e políticas mais ricas podem substituir o middleware em `src/middleware/admin_auth_middleware.rs`.

## Rotas (`/admin`)

| Método | Caminho | Descrição |
|--------|---------|-----------|
| GET | `/admin/dashboard` | Volume, yield acumulado, disputas abertas, custódias travadas |
| GET | `/admin/sellers/{id}/dashboard` | Pedidos, volume, score médio, disputas do vendedor |
| GET | `/admin/orders` | Lista de pedidos (`status`, `min_score`, `from`, `to`) |
| GET | `/admin/disputes` | Todas as disputas |
| GET | `/admin/disputes/{id}` | Detalhe |
| POST | `/admin/disputes/{id}/resolve` | Corpo JSON: `resolution`, `notes` |
| GET | `/admin/reports/yield` | Relatório de rendimento (`from`, `to`) |
| GET | `/admin/users/score` | Scores (`max_score`, `min_risk`) |

## Domínio

Handlers usam `CustodyService`, `DisputeService`, `AntiFraudeService` e o índice em memória de pedidos (`StoredOrder`, alinhado a `apicash-core`). O crate `apicash-events` está disponível como `apicash_admin_backend::events` para integração futura com Pulsar.
