//! Application configuration.

mod app_config;
pub mod oauth_config;

pub use app_config::http_bind_addr;
pub use oauth_config::OAuthConfig;
