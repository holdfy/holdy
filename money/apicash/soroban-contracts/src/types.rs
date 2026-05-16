//! Tipos partilhados: estado do escrow, chaves de armazenamento e erros.

use soroban_sdk::{contracterror, contracttype, Address};

/// Estado do ciclo de vida do escrow (espelha `CustodyStatus` / fluxo de pedido).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Fundos depositados e retidos no contrato.
    Locked = 0,
    /// Comprador confirmou a entrega (ainda não liberou os fundos).
    DeliveryConfirmed = 1,
    /// Comprador confirmou receção — principal e yield distribuídos.
    Released = 2,
    /// Disputa aberta — fundos mantidos até resolução.
    Disputed = 3,
    /// Reembolso ao comprador após disputa.
    Refunded = 4,
}

/// Resolução de disputa (admin).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToSeller = 0,
    RefundBuyer = 1,
}

/// Registo de um pedido em custódia.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowRecord {
    pub buyer: Address,
    pub seller: Address,
    /// Contrato do token (SEP-41), tipicamente Stellar Asset Contract (BRLx, etc.).
    pub token: Address,
    /// Principal em unidades mínimas do token (stroops / 10^decimals).
    pub amount: i128,
    pub status: EscrowStatus,
    /// Ledger de criação (para acréscimo de yield simulado em `release`).
    pub locked_at_ledger: u32,
}

/// Chaves de armazenamento persistente.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    /// Endereço que recebe a fatia de plataforma do yield (20%).
    Platform,
    /// Escrow por identificador de pedido (u64 para simplicidade on-chain).
    Escrow(u64),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    EscrowNotFound = 3,
    InvalidAmount = 4,
    WrongStatus = 5,
    Unauthorized = 6,
    /// Saldo do contrato insuficiente para principal + yield acumulado (mintar yield ao contrato antes do release).
    InsufficientBalance = 7,
    EscrowAlreadyExists = 8,
}
