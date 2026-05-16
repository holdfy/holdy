//! Distribuição do **pool de yield** (não o principal) em 70% vendedor, 10% comprador, 20% plataforma.
//!
//! Mesma política que [`apicash_shared::YIELD_DISTRIBUTION_PERCENT`] e `apicash-custody::split_yield_pool`.

use soroban_sdk::Env;

/// Partes em basis points (soma = 10_000): vendedor 7000, comprador 1000, plataforma 2000.
pub const SELLER_BPS: i128 = 7000;
pub const BUYER_BPS: i128 = 1000;
pub const BPS_DENOM: i128 = 10_000;

/// Divide `pool` (yield acumulado) nas três fatias. Resíduo de arredondamento vai para a plataforma.
pub fn split_yield_pool(_env: &Env, pool: i128) -> (i128, i128, i128) {
    if pool <= 0 {
        return (0, 0, 0);
    }
    let seller = pool.saturating_mul(SELLER_BPS) / BPS_DENOM;
    let buyer = pool.saturating_mul(BUYER_BPS) / BPS_DENOM;
    let platform = pool.saturating_sub(seller).saturating_sub(buyer);
    (seller, buyer, platform)
}

/// Yield simulado: proporcional ao principal e ao número de ledgers decorridos (modelo linear simples).
/// Em produção, substituir por taxa da pool Soroban / oráculo.
pub fn accrued_yield_simple(principal: i128, ledgers_elapsed: u32) -> i128 {
    if principal <= 0 || ledgers_elapsed == 0 {
        return 0;
    }
    // ~0,01% por 100 ledgers (ajustável): (principal * ledgers) / 1_000_000
    principal
        .saturating_mul(ledgers_elapsed as i128)
        .saturating_div(1_000_000)
}
