//! Create order request body.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub buyer_id: Uuid,
    pub seller_id: Uuid,
    /// Decimal string (e.g. `"100.50"`).
    pub amount: String,
    /// CPF digits for anti-fraud (11 digits).
    pub cpf: String,
    pub social_links: Vec<String>,
    /// Descrição do item/serviço (opcional).
    #[serde(default)]
    pub description: Option<String>,
}

impl CreateOrderRequest {
    pub fn validate(&self) -> Result<(), &'static str> {
        let digits: String = self.cpf.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 11 {
            return Err("cpf must have 11 digits");
        }
        if self.amount.trim().is_empty() {
            return Err("amount is required");
        }
        Ok(())
    }
}
