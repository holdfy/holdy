//! Offline mathematical validation for CPF (11 digits) and CNPJ (14 digits).
//!
//! Both algorithms follow Receita Federal specification:
//!   1. Weighted sum of body digits mod 11
//!   2. Remainder < 2 → check digit is 0; else check digit is 11 − remainder
//!
//! No HTTP calls, no secrets required. Suitable as default in all environments.

use async_trait::async_trait;

use crate::error::AntiFraudeError;
use crate::validation::document_validator::{DocumentStatus, DocumentType, DocumentValidator};

pub struct LocalDocumentValidator;

impl LocalDocumentValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalDocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentValidator for LocalDocumentValidator {
    async fn validate(
        &self,
        document: &str,
        doc_type: DocumentType,
    ) -> Result<DocumentStatus, AntiFraudeError> {
        let digits: String = document.chars().filter(|c| c.is_ascii_digit()).collect();
        let valid = match doc_type {
            DocumentType::Cpf => cpf_valid(&digits),
            DocumentType::Cnpj => cnpj_valid(&digits),
        };
        Ok(if valid { DocumentStatus::Valid } else { DocumentStatus::Invalid })
    }
}

// ─── CPF ─────────────────────────────────────────────────────────────────────

fn cpf_valid(digits: &str) -> bool {
    let d: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 11 {
        return false;
    }
    // All-same-digit CPFs are arithmetically valid but officially invalid.
    if d.windows(2).all(|w| w[0] == w[1]) {
        return false;
    }
    weighted_check(&d, 9, 9) && weighted_check(&d, 10, 10)
}

// ─── CNPJ ────────────────────────────────────────────────────────────────────

fn cnpj_valid(digits: &str) -> bool {
    let d: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 14 {
        return false;
    }
    // All-same-digit CNPJs are officially invalid.
    if d.windows(2).all(|w| w[0] == w[1]) {
        return false;
    }
    const W1: [u32; 12] = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    const W2: [u32; 13] = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    cnpj_check(&d, 12, 12, &W1) && cnpj_check(&d, 13, 13, &W2)
}

fn cnpj_check(d: &[u32], body_len: usize, check_idx: usize, weights: &[u32]) -> bool {
    let sum: u32 = d[..body_len].iter().zip(weights.iter()).map(|(&v, &w)| v * w).sum();
    let rem = sum % 11;
    let expected = if rem < 2 { 0 } else { 11 - rem };
    expected == d[check_idx]
}

// ─── Shared helper ───────────────────────────────────────────────────────────

fn weighted_check(d: &[u32], body_len: usize, check_idx: usize) -> bool {
    let sum: u32 = d[..body_len]
        .iter()
        .enumerate()
        .map(|(i, &v)| v * (body_len as u32 + 1 - i as u32))
        .sum();
    let rem = sum % 11;
    let expected = if rem < 2 { 0 } else { 11 - rem };
    expected == d[check_idx]
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_cpf() {
        assert!(cpf_valid("52998224725"));
    }

    #[test]
    fn invalid_cpf_all_zeros() {
        assert!(!cpf_valid("00000000000"));
    }

    #[test]
    fn invalid_cpf_wrong_digits() {
        assert!(!cpf_valid("52998224752"));
    }

    #[test]
    fn valid_cnpj() {
        // 11.222.333/0001-81 — mathematically valid CNPJ
        assert!(cnpj_valid("11222333000181"));
    }

    #[test]
    fn invalid_cnpj_all_zeros() {
        assert!(!cnpj_valid("00000000000000"));
    }

    #[test]
    fn invalid_cnpj_wrong_check() {
        assert!(!cnpj_valid("11222333000199"));
    }
}
