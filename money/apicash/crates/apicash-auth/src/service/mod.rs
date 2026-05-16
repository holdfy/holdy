//! Serviço de autenticação.

mod auth_service;

pub use auth_service::{AuthError, AuthService, LoginRequest, LoginResponse, RefreshRequest};
