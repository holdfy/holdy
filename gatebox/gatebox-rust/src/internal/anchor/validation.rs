use crate::internal::anchor::types::EntityType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnchorValidationError {
    #[error("anchor: entity_type inválido")]
    InvalidEntityType,
    #[error("anchor: entity_id vazio")]
    InvalidEntityId,
    #[error("anchor: account_id deve ser > 0")]
    InvalidAccountId,
}

/// ValidateRequest valida os campos mínimos antes de publicar.
pub fn validate_request(
    entity_type: &EntityType,
    entity_id: &str,
    account_id: i64,
) -> Result<(), AnchorValidationError> {
    if entity_id.is_empty() {
        return Err(AnchorValidationError::InvalidEntityId);
    }
    if account_id <= 0 {
        return Err(AnchorValidationError::InvalidAccountId);
    }
    let _ = entity_type;
    Ok(())
}

/// Valida entity_type como string (para chamadas externas).
pub fn validate_request_str(
    entity_type: &str,
    entity_id: &str,
    account_id: i64,
) -> Result<(), AnchorValidationError> {
    if !EntityType::valid(entity_type) {
        return Err(AnchorValidationError::InvalidEntityType);
    }
    if entity_id.is_empty() {
        return Err(AnchorValidationError::InvalidEntityId);
    }
    if account_id <= 0 {
        return Err(AnchorValidationError::InvalidAccountId);
    }
    Ok(())
}
