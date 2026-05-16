//! Fixed-precision monetary amounts using [`rust_decimal::Decimal`] — never floating point.

use std::fmt;
use std::str::FromStr;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Wrapper around [`Decimal`] for all financial values in APICash.
///
/// Serialization uses string form to preserve precision in JSON.
/// JSON and other serde formats use string encoding for exact decimal semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Money(#[serde(with = "rust_decimal::serde::str")] pub Decimal);

impl Money {
    /// Zero money.
    pub const ZERO: Self = Self(Decimal::ZERO);

    #[inline]
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    #[inline]
    pub fn decimal(&self) -> Decimal {
        self.0
    }

    /// Parse from a decimal string (e.g. `"123.45"`).
    pub fn from_str_strict(s: &str) -> Result<Self, rust_decimal::Error> {
        Ok(Self(Decimal::from_str_exact(s)?))
    }

    #[inline]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    #[inline]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    #[inline]
    pub fn is_positive(&self) -> bool {
        self.0.is_sign_positive() && !self.0.is_zero()
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Decimal> for Money {
    fn from(value: Decimal) -> Self {
        Self(value)
    }
}

impl From<Money> for Decimal {
    fn from(value: Money) -> Self {
        value.0
    }
}

impl FromStr for Money {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str_exact(s).map(Self)
    }
}
