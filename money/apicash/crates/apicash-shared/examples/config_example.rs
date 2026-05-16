//! Demonstrates loading [`apicash_shared::AppConfig`] from the environment.
//!
//! Run (from workspace root):
//! `cargo run -p apicash-shared --example config_example`

use apicash_shared::AppConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Minimal env for `AppConfig::load()` — in real deployments set these in the shell or `.env`.
    std::env::set_var("APICASH__ENV", "development");
    std::env::set_var(
        "APICASH__DATABASE__URL",
        "postgresql://apicash:apicash@localhost:5432/apicash",
    );
    std::env::set_var("APICASH__DATABASE__MAX_CONNECTIONS", "10");
    std::env::set_var("APICASH__REDIS__URL", "redis://127.0.0.1:6379/0");
    std::env::set_var("APICASH__STELLAR__NETWORK", "testnet");
    std::env::set_var(
        "APICASH__STELLAR__HORIZON_URL",
        "https://horizon-testnet.stellar.org",
    );
    std::env::set_var(
        "APICASH__STELLAR__RPC_URL",
        "https://soroban-testnet.stellar.org",
    );
    std::env::set_var("APICASH__PULSAR__SERVICE_URL", "pulsar://localhost:6650");
    std::env::set_var("APICASH__PULSAR__TENANT", "apicash");
    std::env::set_var("APICASH__PULSAR__NAMESPACE", "default");
    std::env::set_var("APICASH__WHATSAPP__TOKEN", "");
    std::env::set_var("APICASH__WHATSAPP__PHONE_NUMBER_ID", "");
    std::env::set_var("APICASH__WHATSAPP__BUSINESS_ACCOUNT_ID", "");
    std::env::set_var("APICASH__WHATSAPP__WEBHOOK_VERIFY_TOKEN", "");
    std::env::set_var("APICASH__ANTIFRAUDE__ENABLED", "false");
    std::env::set_var("APICASH__AUTH__JWT_ISSUER", "apicash");
    std::env::set_var("APICASH__AUTH__JWT_AUDIENCE", "apicash-api");
    std::env::set_var("APICASH__AUTH__JWT_SECRET", "dev-only-change-me");
    std::env::set_var("APICASH__NOTIFICATIONS__SMTP_HOST", "");
    std::env::set_var("APICASH__NOTIFICATIONS__SMS_PROVIDER_API_KEY", "");

    let cfg = AppConfig::load()?;
    println!("Loaded APICash config: env={}", cfg.env);
    Ok(())
}
