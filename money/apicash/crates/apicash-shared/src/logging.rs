//! Inicialização única de [`tracing_subscriber`] para binários (texto ou JSON em produção).

/// Inicializa logging com filtro por ambiente (`RUST_LOG`) ou `default_filter`.
///
/// Com `APICASH_LOG_FORMAT=json` (ou `true`), emite linhas JSON estruturadas (adequado a agregadores).
/// Chamar apenas uma vez no `main` de cada binário.
pub fn init_tracing(default_filter: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_filter));
    let json = std::env::var("APICASH_LOG_FORMAT")
        .map(|v| v == "json" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}
