//! Binário `apicash` — ferramentas de desenvolvimento e testes manuais.

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;
mod utils;

#[derive(Parser, Debug)]
#[command(
    name = "apicash-cli",
    version,
    about = "APICash — CLI interna (fluxos, custódia, score, Stellar testnet)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Simula fluxo completo: pedido → PIX → custódia → libertação (HTTP contra `apicash-core`).
    TestFlow,
    /// Consulta estado do pedido/custódia via API pública.
    CheckCustody { order_id: uuid::Uuid },
    /// Lista scores de utilizadores via admin API (ou placeholder se admin indisponível).
    SimulateScore { cpf: String },
    /// Consulta saldo de conta na Stellar testnet (Horizon).
    StellarBalance {
        /// Conta Stellar (G...). Se omitido, usa `APICASH_STELLAR_TEST_ACCOUNT` ou placeholder.
        #[arg(long)]
        account: Option<String>,
    },
    /// Deploy do contrato Soroban de escrow na testnet (`stellar contract deploy`).
    DeployContracts,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::TestFlow => commands::test_flow::run().await,
        Commands::CheckCustody { order_id } => commands::check_custody::run(order_id).await,
        Commands::SimulateScore { cpf } => commands::simulate_score::run(&cpf).await,
        Commands::StellarBalance { account } => {
            utils::stellar_testnet::print_balance(account.as_deref()).await
        }
        Commands::DeployContracts => commands::deploy_contracts::run().await,
    }
}
