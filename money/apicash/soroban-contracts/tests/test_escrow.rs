//! Testes de integração do contrato de escrow com Stellar Asset Contract (testnet local).

use apicash_soroban_contracts::{
    accrued_yield_simple, split_yield_pool, EscrowContract, EscrowContractClient,
};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{Address, Env};

#[test]
fn yield_split_matches_70_10_20() {
    let env = Env::default();
    let pool: i128 = 10_000;
    let (seller, buyer, platform) = split_yield_pool(&env, pool);
    assert_eq!(seller, 7000);
    assert_eq!(buyer, 1000);
    assert_eq!(platform, 2000);
    assert_eq!(seller + buyer + platform, pool);
}

#[test]
fn escrow_lock_confirm_release_distributes_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let platform = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_addr = sac.address();
    let stellar = StellarAssetClient::new(&env, &token_addr);
    let token = TokenClient::new(&env, &token_addr);

    let principal: i128 = 1_000_000;
    stellar.mint(&buyer, &principal);

    let escrow_addr = env.register(EscrowContract, ());
    let escrow = EscrowContractClient::new(&env, &escrow_addr);

    escrow.init(&admin, &platform);

    // Realistic flow: BRLx is transferred to the escrow contract address first (post on-ramp),
    // then the escrow record is locked on-chain.
    token.transfer(&buyer, &escrow_addr, &principal);

    escrow.lock_funds(&1_u64, &buyer, &seller, &token_addr, &principal);

    assert_eq!(token.balance(&buyer), 0);
    assert_eq!(token.balance(&escrow_addr), principal);

    let ledgers = 500_000_u32;
    env.ledger()
        .set_sequence_number(env.ledger().sequence() + ledgers);

    let yield_pool = accrued_yield_simple(principal, ledgers);
    assert!(yield_pool > 0);
    stellar.mint(&escrow_addr, &yield_pool);

    escrow.confirm_release(&1_u64);

    let (s_y, b_y, p_y) = split_yield_pool(&env, yield_pool);

    assert_eq!(token.balance(&escrow_addr), 0);
    assert_eq!(token.balance(&seller), principal + s_y);
    assert_eq!(token.balance(&buyer), b_y);
    assert_eq!(token.balance(&platform), p_y);
}
