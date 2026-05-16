//! Entry point do agente WhatsApp (serviço separado da API `apicash-core`).

use std::path::Path;

use apicash_whatsapp::WhatsAppService;

fn load_workspace_dotenv() {
    let money_env = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.env");
    if money_env.is_file() {
        let _ = dotenvy::from_path(&money_env);
    }
    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::dotenv();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    load_workspace_dotenv();
    apicash_shared::logging::init_tracing("info,apicash_whatsapp=info");

    let _svc = WhatsAppService::start().await?;

    tracing::info!(
        "HoldFy WhatsApp agent em execução (transporte por defeito: whatsapp-rust; ver APICASH_WA_TRANSPORT)"
    );

    tokio::signal::ctrl_c().await?;
    tracing::info!("encerrando");
    Ok(())
}
