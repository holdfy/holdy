//! Cross-cutting HTTP middleware.

mod auth_middleware;

pub use auth_middleware::auth_middleware;
