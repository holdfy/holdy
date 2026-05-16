//! Yield accrual and 70/10/20 distribution of **yield** (principal handled at release policy).

mod distribution;
mod yield_calculator;

pub use distribution::{ratios_sum_to_one, split_yield_pool};
pub use yield_calculator::YieldCalculator;
