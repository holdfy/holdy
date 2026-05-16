//! Contrato de custódia: bloqueia token SEP-41 no escrow e libera com split de yield (70/10/20).

use soroban_sdk::{contract, contractimpl, token::TokenClient, Address, Env};

use crate::types::{DataKey, DisputeResolution, EscrowError, EscrowRecord, EscrowStatus};
use crate::yield_distributor::{accrued_yield_simple, split_yield_pool};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Alias explícito para `init` (nome esperado por integrações).
    pub fn initialize(env: Env, admin: Address, platform: Address) -> Result<(), EscrowError> {
        Self::init(env, admin, platform)
    }

    /// Inicializa admin e endereço da plataforma (recebe 20% do yield).
    pub fn init(env: Env, admin: Address, platform: Address) -> Result<(), EscrowError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(EscrowError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Platform, &platform);
        Ok(())
    }

    /// Alias explícito para `lock_funds` (nome esperado por integrações).
    pub fn lock(
        env: Env,
        order_id: u64,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), EscrowError> {
        Self::lock_funds(env, order_id, buyer, seller, token, amount)
    }

    /// Comprador deposita `amount` do `token` no contrato e cria escrow `order_id`.
    pub fn lock_funds(
        env: Env,
        order_id: u64,
        buyer: Address,
        seller: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), EscrowError> {
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }
        buyer.require_auth();

        if env.storage().persistent().has(&DataKey::Escrow(order_id)) {
            return Err(EscrowError::EscrowAlreadyExists);
        }

        // Accept both flows: pre-funded escrow contracts from an off-chain anchor and direct
        // buyer deposits when the buyer signs this invocation.
        let contract = env.current_contract_address();
        let token_client = TokenClient::new(&env, &token);
        let bal = token_client.balance(&contract);
        if bal < amount {
            token_client.transfer(&buyer, &contract, &(amount - bal));
        }

        let rec = EscrowRecord {
            buyer,
            seller,
            token,
            amount,
            status: EscrowStatus::Locked,
            locked_at_ledger: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(order_id), &rec);
        Ok(())
    }

    /// Comprador confirma a entrega (passo explícito antes de liberar fundos).
    pub fn confirm_delivery(env: Env, order_id: u64) -> Result<(), EscrowError> {
        let rec: EscrowRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(order_id))
            .ok_or(EscrowError::EscrowNotFound)?;

        if rec.status != EscrowStatus::Locked {
            return Err(EscrowError::WrongStatus);
        }
        rec.buyer.require_auth();

        let mut updated = rec;
        updated.status = EscrowStatus::DeliveryConfirmed;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(order_id), &updated);
        Ok(())
    }

    /// Libera fundos: transfere principal ao vendedor e distribui o pool de yield.
    ///
    /// Requer que o comprador tenha confirmado a entrega via [`confirm_delivery`].
    /// O contrato deve deter saldo ≥ `amount` + `yield_pool` (mintar o yield ao contrato antes do release).
    pub fn release(env: Env, order_id: u64) -> Result<(), EscrowError> {
        let rec: EscrowRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(order_id))
            .ok_or(EscrowError::EscrowNotFound)?;

        if rec.status != EscrowStatus::DeliveryConfirmed {
            return Err(EscrowError::WrongStatus);
        }
        rec.buyer.require_auth();

        let platform: Address = env
            .storage()
            .instance()
            .get(&DataKey::Platform)
            .ok_or(EscrowError::NotInitialized)?;

        let contract = env.current_contract_address();
        let token_client = TokenClient::new(&env, &rec.token);

        let bal = token_client.balance(&contract);
        if bal < rec.amount {
            return Err(EscrowError::InsufficientBalance);
        }

        let ledgers = env.ledger().sequence().saturating_sub(rec.locked_at_ledger);
        let available_yield = bal.saturating_sub(rec.amount);
        let yield_pool = accrued_yield_simple(rec.amount, ledgers).min(available_yield);
        let (seller_yield, buyer_yield, platform_yield) = split_yield_pool(&env, yield_pool);

        token_client.transfer(&contract, &rec.seller, &rec.amount);

        if yield_pool > 0 {
            token_client.transfer(&contract, &rec.seller, &seller_yield);
            token_client.transfer(&contract, &rec.buyer, &buyer_yield);
            token_client.transfer(&contract, &platform, &platform_yield);
        }

        let mut updated = rec;
        updated.status = EscrowStatus::Released;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(order_id), &updated);
        Ok(())
    }

    /// Compatibilidade: confirma e libera em um passo (fluxo legado).
    pub fn confirm_release(env: Env, order_id: u64) -> Result<(), EscrowError> {
        // Keep behavior close to legacy: buyer confirms and releases in one transaction.
        Self::confirm_delivery(env.clone(), order_id)?;
        Self::release(env, order_id)
    }

    /// Abre disputa (buyer ou seller).
    pub fn open_dispute(env: Env, order_id: u64, opened_by: Address) -> Result<(), EscrowError> {
        let mut rec: EscrowRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(order_id))
            .ok_or(EscrowError::EscrowNotFound)?;

        if !matches!(
            rec.status,
            EscrowStatus::Locked | EscrowStatus::DeliveryConfirmed
        ) {
            return Err(EscrowError::WrongStatus);
        }

        opened_by.require_auth();
        if opened_by != rec.buyer && opened_by != rec.seller {
            return Err(EscrowError::Unauthorized);
        }

        rec.status = EscrowStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(order_id), &rec);
        Ok(())
    }

    /// Resolve disputa (admin).
    pub fn resolve_dispute(
        env: Env,
        order_id: u64,
        resolution: DisputeResolution,
    ) -> Result<(), EscrowError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(EscrowError::NotInitialized)?;
        admin.require_auth();

        let rec: EscrowRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(order_id))
            .ok_or(EscrowError::EscrowNotFound)?;

        if rec.status != EscrowStatus::Disputed {
            return Err(EscrowError::WrongStatus);
        }

        let contract = env.current_contract_address();
        let token_client = TokenClient::new(&env, &rec.token);

        // Simple dispute semantics:
        // - ReleaseToSeller: send principal to seller (yield split uses platform rules).
        // - RefundBuyer: refund principal (+ any simulated yield) to buyer.
        let ledgers = env.ledger().sequence().saturating_sub(rec.locked_at_ledger);
        let yield_pool = accrued_yield_simple(rec.amount, ledgers);

        match resolution {
            DisputeResolution::ReleaseToSeller => {
                // Reuse the same distribution logic as normal release.
                let platform: Address = env
                    .storage()
                    .instance()
                    .get(&DataKey::Platform)
                    .ok_or(EscrowError::NotInitialized)?;
                let (seller_yield, buyer_yield, platform_yield) =
                    split_yield_pool(&env, yield_pool);

                let need = rec.amount.saturating_add(yield_pool);
                let bal = token_client.balance(&contract);
                if bal < need {
                    return Err(EscrowError::InsufficientBalance);
                }

                token_client.transfer(&contract, &rec.seller, &rec.amount);
                if yield_pool > 0 {
                    token_client.transfer(&contract, &rec.seller, &seller_yield);
                    token_client.transfer(&contract, &rec.buyer, &buyer_yield);
                    token_client.transfer(&contract, &platform, &platform_yield);
                }

                let mut updated = rec;
                updated.status = EscrowStatus::Released;
                env.storage()
                    .persistent()
                    .set(&DataKey::Escrow(order_id), &updated);
                Ok(())
            }
            DisputeResolution::RefundBuyer => {
                let need = rec.amount.saturating_add(yield_pool);
                let bal = token_client.balance(&contract);
                if bal < need {
                    return Err(EscrowError::InsufficientBalance);
                }
                token_client.transfer(
                    &contract,
                    &rec.buyer,
                    &rec.amount.saturating_add(yield_pool),
                );

                let mut updated = rec;
                updated.status = EscrowStatus::Refunded;
                env.storage()
                    .persistent()
                    .set(&DataKey::Escrow(order_id), &updated);
                Ok(())
            }
        }
    }

    /// Marca escrow em disputa (admin / oráculo).
    pub fn mark_disputed(env: Env, order_id: u64) -> Result<(), EscrowError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(EscrowError::NotInitialized)?;
        admin.require_auth();

        let mut rec: EscrowRecord = env
            .storage()
            .persistent()
            .get(&DataKey::Escrow(order_id))
            .ok_or(EscrowError::EscrowNotFound)?;
        if !matches!(
            rec.status,
            EscrowStatus::Locked | EscrowStatus::DeliveryConfirmed
        ) {
            return Err(EscrowError::WrongStatus);
        }
        rec.status = EscrowStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::Escrow(order_id), &rec);
        Ok(())
    }
}
