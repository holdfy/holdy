use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partners {
    pub id: i64,
    pub partners_list_id: i64,
    pub description: String,
    pub document: String,
    pub account: String,
    pub branch: String,
    pub authentication_id: i64,
    pub client_id: String,
    pub client_secret: String,
    pub authentication: String,
    pub password: String,
    pub whpix_in_id: String,
    pub whpix_out_id: String,
    pub type_authorize_id: i64,
    pub fixed_cash_in: Decimal,
    pub fixed_cash_out: Decimal,
    pub percent_cashin: Decimal,
    pub percent_cashout: Decimal,
    pub fixed_ref_cash_in: Decimal,
    pub fixed_ref_cash_out: Decimal,
    pub percent_ref_cashin: Decimal,
    pub percent_ref_cashout: Decimal,
    pub active: bool,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
