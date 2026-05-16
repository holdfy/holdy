//! HTTP server and wiring configuration from environment.

/// Bind address (default `0.0.0.0:3000`).
pub fn http_bind_addr() -> String {
    let port = std::env::var("APICASH_HTTP_PORT").unwrap_or_else(|_| "3000".into());
    std::env::var("APICASH_HTTP_BIND").unwrap_or_else(|_| format!("0.0.0.0:{port}"))
}
