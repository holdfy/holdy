# apicash-frontend

Dashboard administrativo HoldFy (Leptos: SSR no binário nativo + hydration WASM).

## Variáveis de ambiente (servidor SSR)

| Variável | Descrição |
|----------|-----------|
| `APICASH_ADMIN_API_URL` | Base URL do `apicash-admin-backend` (default `http://127.0.0.1:3001`). |
| `APICASH_ADMIN_API_KEY` | Chave para o header `x-api-key` nas chamadas ao backend. |

O endereço do site Leptos está em `Cargo.toml` → `[package.metadata.leptos]` (`site-addr`, default `127.0.0.1:3002`).

## Executar (SSR)

```bash
cd apicash
cargo run -p apicash-frontend --features ssr
```

## Build WASM (hydrate)

```bash
cargo build -p apicash-frontend --no-default-features --features hydrate --target wasm32-unknown-unknown
```

Para o fluxo completo (assets + WASM), use `cargo leptos` conforme `Cargo.toml.leptos` na raiz do crate.

## Features

- `ssr`: servidor Axum + server functions que chamam o admin backend (inclui `reqwest`).
- `hydrate`: cliente WASM + função `hydrate()` em `lib.rs` (sem `reqwest` no bundle).

O default do crate é `ssr` + `hydrate` para desenvolvimento local; o alvo WASM em produção costuma usar só `hydrate`.
