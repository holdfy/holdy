use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fees {
    pub id: i64,
    pub account_id: i64,
    pub fixed_cash_in: Decimal,
    pub fixed_cash_out: Decimal,
    pub percent_cashin: Decimal,
    pub percent_cashout: Decimal,
    pub percentsec_med: Decimal,
    pub fixed_ref_cash_in: Decimal,
    pub fixed_ref_cash_out: Decimal,
    pub apagar: String,
    pub percent_ref_cashin: Decimal,
    pub percent_ref_cashout: Decimal,
    pub deleted_at: Option<DateTime<Utc>>,
    /// 1 = NATURAL_PERSON (PF), 2 = LEGAL_PERSON (PJ), NULL = any type.
    pub person_type_id: Option<i64>,
    #[serde(skip_serializing)]
    pub full_count: Option<i64>,
}
