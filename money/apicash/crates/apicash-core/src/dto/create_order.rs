//! Create order request body.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    /// Decimal string (e.g. `"100.50"`).
    pub amount: String,
    /// CPF (11 digits) or CNPJ (14 digits) for anti-fraud.
    pub cpf: String,
    pub social_links: Vec<String>,
    /// Descrição do item/serviço (opcional).
    #[serde(default)]
    pub description: Option<String>,
    /// Nome completo do comprador (opcional, obtido via NFS-e).
    #[serde(default)]
    pub buyer_name: Option<String>,
}

impl CreateOrderRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        let digits: String = self.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 11 && digits.len() != 14 {
            return Err("cpf must have 11 digits (CPF) or 14 digits (CNPJ)");
        }
        if self.amount.trim().is_empty() {
            return Err("amount is required");
        }
        Ok(())
    }
}
