//! Anti-fraud: user score, SEFAZ/Receita checks, and social validation before Stellar on-ramp.
//!
//! Exposição na API:
//! - `apicash-core` oferece `POST /risk/score` (protegido por JWT do usuário final).
//! - Para integrações internas (ex.: WhatsApp Agent), `apicash-core` também expõe
//!   `POST /internal/risk/score` protegido por chave de serviço (`X-API-Key` = `APICASH_API_KEY`).

pub use rust_decimal::Decimal;

pub mod error;
pub mod models;
pub mod repository;
pub mod score;
pub mod service;
pub mod validation;

pub use crate::error::AntiFraudeError;
pub use crate::repository::{InMemoryScoreRepository, PostgresScoreRepository, ScoreRepository};
pub use crate::score::{OnRampDecision, RiskFactor, RiskLevel, ScoreCalculator, UserScore};
pub use crate::service::AntiFraudeService;
pub use crate::validation::{
    SefazPersonStatus, SefazValidationResult, SefazValidator, SocialAccountSnapshot,
    SocialValidationResult, SocialValidator,
};
