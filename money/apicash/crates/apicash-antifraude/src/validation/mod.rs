//! Document and social validation adapters.

mod cached_document_validator;
mod document_cache;
mod document_validator;
mod http_document_validator;
mod local_document_validator;
mod social_validator;
mod validation_result;

pub use cached_document_validator::CachedDocumentValidator;
pub use document_cache::{DocumentCache, InMemoryDocumentCache, PostgresDocumentCache};
pub use document_validator::{DocumentStatus, DocumentType, DocumentValidator};
pub use http_document_validator::HttpDocumentValidator;
pub use local_document_validator::LocalDocumentValidator;
pub use social_validator::SocialValidator;
pub use validation_result::{SocialAccountSnapshot, SocialValidationResult};
