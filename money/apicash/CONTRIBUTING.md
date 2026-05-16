# Contribuir para o APICash

Obrigado pelo interesse em melhorar o projeto.

## Ambiente

- **Rust** 1.85+ (`rustup update stable`)
- **Docker** opcional para Postgres, Redis e Pulsar locais
- Na raiz do workspace: `cd apicash`

## Fluxo sugerido

1. Crie um branch a partir do estado atual do repositório principal.
2. Execute `cargo fmt --all` e `cargo clippy --workspace --all-targets` antes de abrir PR.
3. Execute `cargo test --workspace` (ou `make test-all`).
4. Descreva no PR o **comportamento**, o **porquê** da mudança e qualquer decisão de compatibilidade (API, env).

## Convenções

- **Erros** entre crates: preferir [`ApiCashError`](crates/apicash-shared/src/error/apicash_error.rs) em superfícies públicas quando fizer sentido.
- **Valores monetários**: [`Money`](crates/apicash-shared/src/types/money.rs) / `rust_decimal::Decimal`, não `f64` em lógica de negócio.
- **Observabilidade**: `tracing` (`info` / `warn` / `error`) em caminhos de serviço e falhas externas.
- **Documentação de API**: manter `docs/arquitetura.md` e o README alinhados com mudanças visíveis.

## Commits

Mensagens claras em português ou inglês; uma ideia por commit quando possível.
