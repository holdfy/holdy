//! Autenticação e segurança APICash: JWT (HS256), claims com papéis, middleware Axum.
//!
//! Integração opcional com [`apicash_antifraude`] (`feature = "antifraude"`, default): o score
//! pode ser embutido no token e usado em [`JwtClaims::can_operate_high_risk`].

mod auth_error_map;

pub mod config;
pub mod middleware;
pub mod models;
pub mod service;

pub use config::AuthConfig;
pub use models::claims::{JwtClaims, Role};
pub use models::session::SessionInfo;
pub use models::user::UserIdentity;
pub use service::{AuthError, AuthService, LoginRequest, LoginResponse, RefreshRequest};
