//! Deploy do Wasm do escrow na Stellar testnet via CLI `stellar` (Stellar CLI).

use anyhow::{Context, Result};

pub async fn run() -> Result<()> {
    // Best-effort build before deploy (does not force deploy if tooling isn't ready).
    let wasm = std::env::var("APICASH_SOROBAN_WASM_PATH")
        .unwrap_or_else(|_| "target/wasm32v1-none/release/apicash_soroban_contracts.wasm".into());
    let rpc = std::env::var("APICASH_SOROBAN_RPC_URL")
        .or_else(|_| std::env::var("STELLAR_RPC_URL"))
        .context("defina APICASH_SOROBAN_RPC_URL (ex.: https://soroban-testnet.stellar.org)")?;
    let passphrase = std::env::var("APICASH_STELLAR_NETWORK_PASSPHRASE")
        .unwrap_or_else(|_| "Test SDF Network ; September 2015".into());
    let source = std::env::var("APICASH_SOROBAN_SOURCE_SECRET")
        .or_else(|_| std::env::var("APICASH_STELLAR_SECRET_KEY"))
        .context("defina APICASH_SOROBAN_SOURCE_SECRET (chave de assinatura da conta deployer)")?;
    let bin = std::env::var("APICASH_STELLAR_CLI_BIN").unwrap_or_else(|_| "stellar".into());

    // Attempt build via `stellar contract build` if available; fall back to current file if it already exists.
    {
        let mut build = tokio::process::Command::new(&bin);
        build.args(["contract", "build"]);
        build.current_dir("soroban-contracts");
        tracing::info!("deploy-contracts: tentando build do contrato (`stellar contract build`)");
        match build.output().await {
            Ok(out) if out.status.success() => {
                tracing::info!("deploy-contracts: build ok");
            }
            Ok(out) => {
                tracing::warn!(
                    stderr = %String::from_utf8_lossy(&out.stderr),
                    "deploy-contracts: build falhou (continuando para deploy se wasm existir)"
                );
            }
            Err(e) => {
                tracing::warn!(error = %e, "deploy-contracts: build não executado");
            }
        }
    }

    tracing::info!(wasm = %wasm, rpc = %rpc, "deploy-contracts: stellar contract deploy");

    let mut cmd = tokio::process::Command::new(&bin);
    cmd.args([
        "contract",
        "deploy",
        "--wasm",
        &wasm,
        "--rpc-url",
        &rpc,
        "--network-passphrase",
        &passphrase,
        "--source",
        &source,
    ]);
    let out = cmd
        .output()
        .await
        .with_context(|| format!("executar `{bin} contract deploy`"))?;
    if !out.status.success() {
        anyhow::bail!(
            "stellar deploy falhou: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    println!("{stdout}");
    let contract_id = stdout
        .lines()
        .find(|l| l.trim_start().starts_with('C') && l.trim().len() >= 50)
        .map(str::trim)
        .unwrap_or_else(|| stdout.trim());

    if env_truthy("APICASH_SOROBAN_INIT") {
        let admin = std::env::var("APICASH_SOROBAN_ADMIN_ADDRESS")
            .or_else(|_| std::env::var("APICASH_STELLAR_ADMIN_ADDRESS"))
            .context("APICASH_SOROBAN_INIT=1 requer APICASH_SOROBAN_ADMIN_ADDRESS")?;
        let platform = std::env::var("APICASH_SOROBAN_PLATFORM_ADDRESS")
            .or_else(|_| std::env::var("APICASH_STELLAR_PLATFORM_ADDRESS"))
            .context("APICASH_SOROBAN_INIT=1 requer APICASH_SOROBAN_PLATFORM_ADDRESS")?;

        let mut init = tokio::process::Command::new(&bin);
        init.args([
            "contract",
            "invoke",
            "--id",
            contract_id,
            "--rpc-url",
            &rpc,
            "--network-passphrase",
            &passphrase,
            "--source",
            &source,
            "--",
            "initialize",
            "--admin",
            &admin,
            "--platform",
            &platform,
        ]);
        let init_out = init
            .output()
            .await
            .with_context(|| format!("executar `{bin} contract invoke initialize`"))?;
        if !init_out.status.success() {
            anyhow::bail!(
                "stellar initialize falhou: {}",
                String::from_utf8_lossy(&init_out.stderr)
            );
        }
        println!("{}", String::from_utf8_lossy(&init_out.stdout));
    }

    println!("APICASH_SOROBAN_ESCROW_CONTRACT_ID={contract_id}");
    tracing::info!("deploy-contracts: concluído");
    Ok(())
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}
