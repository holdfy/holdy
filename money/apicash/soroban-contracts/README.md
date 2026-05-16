# APICash — contratos Soroban (custódia + yield)

Crate Rust com contratos **Soroban** para a Fase 2 do APICash: **escrow** on-chain (token SEP-41) e **distribuição de yield** alinhada com `apicash-custody` (**70% vendedor**, **10% comprador**, **20% plataforma** sobre o *pool de yield*, não sobre o principal).

## Estrutura

| Ficheiro | Conteúdo |
|----------|------------|
| `src/escrow.rs` | `EscrowContract`: `init`, `lock_funds`, `confirm_release`, `mark_disputed` |
| `src/yield_distributor.rs` | `split_yield_pool`, `accrued_yield_simple` (modelo linear para testes) |
| `src/types.rs` | `EscrowRecord`, `EscrowStatus`, `DataKey`, `EscrowError` |
| `tests/test_escrow.rs` | Integração com Stellar Asset Contract no `Env` de testes |

## Requisitos

- Rust **1.85+** (alinhado ao workspace APICash)
- `soroban-sdk` **23.5** (via `[workspace.dependencies]`)

## Testes

Na raiz do workspace APICash (`apicash/`):

```bash
cargo test -p apicash-soroban-contracts
```

## Build Wasm (deploy)

Use a CLI Stellar/Soroban apontando para este diretório:

```bash
rustup target add wasm32v1-none
stellar contract build
```

O artefato de deploy fica em `target/wasm32v1-none/release/apicash_soroban_contracts.wasm` na raiz do workspace.

## Fluxo on-chain (resumo)

1. **`init`** — define `admin` e endereço da **plataforma** (recebe 20% do yield).
2. **`lock_funds`** — o comprador autoriza o escrow; se o contrato ainda não estiver pré-fundado pelo anchor/off-chain, o próprio contrato transfere o **principal** do comprador para o escrow.
3. Antes de **`confirm_release`**, o contrato deve deter saldo ≥ **principal**; o *yield* acumulado é calculado em contrato (`accrued_yield_simple`) e limitado ao excedente realmente disponível no contrato.
4. **`confirm_release`** — o comprador confirma; o contrato envia o principal ao vendedor e reparte o *yield* 70/10/20.

## Integração com o backend

O serviço `apicash-custody` continua a ser a fonte de verdade off-chain até à ligação RPC/Soroban; este crate fornece a **lógica equivalente** para *deploy* e *invoke* na rede Stellar (testnet / futurenet / mainnet conforme configuração).
