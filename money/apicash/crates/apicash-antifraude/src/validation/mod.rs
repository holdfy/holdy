//! External validation adapters (SEFAZ / social).

mod sefaz_validator;
mod social_validator;
mod validation_result;

pub use sefaz_validator::SefazValidator;
pub use social_validator::SocialValidator;
pub use validation_result::{
    SefazPersonStatus, SefazValidationResult, SocialAccountSnapshot, SocialValidationResult,
};
