//! APICash REST API entrypoint.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use apicash_core::{create_router, AppState};
use apicash_shared::OrderStatus;
use chrono::Utc;

/// Carrega `money/.env` de forma fiável: o processo corre normalmente com CWD `apicash/`, onde só
/// `dotenv()` nem sempre bastava; sem `GATEBOX_BASE_URL` o rail simulado não consegue obter PIX.
fn load_workspace_dotenv() {
    let money_env = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.env");
    if money_env.is_file() {
        if let Err(e) = dotenvy::from_path(&money_env) {
            eprintln!("apicash-core: aviso ao ler {}: {e}", money_env.display());
        }
    }
    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::dotenv();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    load_workspace_dotenv();

    #[cfg(not(feature = "soroban"))]
    if apicash_shared::require_testnet() {
        eprintln!(
            "apicash-core: APICASH_REQUIRE_TESTNET=1 exige compilação com --features soroban (./runapp.sh build apicash)"
        );
        std::process::exit(1);
    }

    if let Err(msg) = apicash_shared::assert_testnet_live_config() {
        eprintln!("{msg}");
        std::process::exit(1);
    }

    apicash_shared::logging::init_tracing(
        "info,apicash_core=info,tower_http=info,tower_governor=warn",
    );

    if std::env::var("GATEBOX_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .is_some()
    {
        tracing::info!(
            "money/.env: GATEBOX_BASE_URL definido — rail simulado usará POST Gatebox /api/v1/pix/qrcode (PIX copia-e-cola real)"
        );
    } else if matches!(
        std::env::var("APICASH_FIAT_RAIL").as_deref(),
        Ok("simulated") | Ok("mock")
    ) {
        tracing::warn!(
            "GATEBOX_BASE_URL ausente — com APICASH_FIAT_RAIL=simulated a criação de pedido falha até configurar Gatebox em money/.env"
        );
    }

    let state = Arc::new(AppState::connect_from_env().await?);
    if env_enabled("APICASH_FUNDING_POLLER") {
        let poller_state = state.clone();
        tokio::spawn(async move {
            run_funding_poller(poller_state).await;
        });
    }
    let app = create_router(state);

    let addr = apicash_core::config::http_bind_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "apicash-core listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

async fn run_funding_poller(state: Arc<AppState>) {
    tracing::info!("funding poller enabled");
    let interval = std::time::Duration::from_secs(10);
    loop {
        let orders = match state.orders.list_all().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "funding poller: list_all failed");
                tokio::time::sleep(interval).await;
                continue;
            }
        };
        for mut stored in orders {
            if stored.order.status != OrderStatus::PendingFunding {
                continue;
            }
            let expiry_mins = std::env::var("APICASH_FUNDING_EXPIRY_MINUTES")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(30);
            if stored.order.created_at + chrono::Duration::minutes(expiry_mins) < Utc::now() {
                stored.order.status = OrderStatus::Failed;
                stored.order.updated_at = Utc::now();
                if let Err(e) = state.orders.update(stored).await {
                    tracing::warn!(error = %e, "funding poller: failed to mark expired order");
                }
                continue;
            }
            let Some(tx_id) = stored.gateway_in_tx_id.clone() else {
                continue;
            };
            match state
                .anchor
                .poll_funding_settlement(&tx_id, stored.funding_reference.as_deref())
                .await
            {
                Ok(settle) if settle.settled => {
                    let escrow_addr = match std::env::var("APICASH_SOROBAN_ESCROW_CONTRACT_ID") {
                        Ok(v) if !v.trim().is_empty() && !v.contains("mock") => v,
                        _ if apicash_shared::require_testnet() => {
                            tracing::warn!(
                                order_id = %stored.order.id,
                                "funding poller: APICASH_SOROBAN_ESCROW_CONTRACT_ID missing — skip until testnet escrow configured"
                            );
                            continue;
                        }
                        _ => "mock_escrow_contract".into(),
                    };
                    let memo = stored
                        .funding_reference
                        .clone()
                        .unwrap_or_else(|| format!("order:{}", stored.order.id));
                    let transferred = match state
                        .anchor
                        .transfer_brlx_to_escrow(&escrow_addr, stored.order.amount, &memo)
                        .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!(order_id = %stored.order.id, error = %e, "poller transfer failed");
                            continue;
                        }
                    };
                    if apicash_shared::require_testnet() && transferred.is_mock {
                        tracing::warn!(
                            order_id = %stored.order.id,
                            "poller: BRLx transfer was mock — APICASH_REQUIRE_TESTNET requires real testnet tx"
                        );
                        continue;
                    }
                    let custody = match state.custody.lock_funds(&stored.order).await {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!(order_id = %stored.order.id, error = %e, "poller lock failed");
                            continue;
                        }
                    };
                    stored.order.status = OrderStatus::InCustody;
                    stored.order.updated_at = Utc::now();
                    stored.custody_id = Some(custody.id);
                    stored.anchor_tx_hash = stored
                        .anchor_tx_hash
                        .clone()
                        .or_else(|| settle.transaction_id.clone());
                    stored.brlx_escrow_transfer_tx_hash = Some(transferred.tx_hash);
                    stored.soroban_escrow_contract_id = custody.soroban_escrow_contract_id;
                    stored.soroban_lock_tx_hash = custody.soroban_lock_tx_hash;
                    stored.soroban_mode = if custody.soroban_is_mock {
                        "mock".into()
                    } else {
                        "soroban".into()
                    };
                    if let Err(e) = state.orders.update(stored).await {
                        tracing::warn!(error = %e, "funding poller: update failed");
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(order_id = %stored.order.id, error = %e, "funding poller check failed")
                }
            }
        }
        tokio::time::sleep(interval).await;
    }
}

fn env_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => tracing::info!("shutdown signal received, finishing in-flight requests"),
        Err(e) => tracing::error!(error = %e, "failed to listen for shutdown signal"),
    }
}
