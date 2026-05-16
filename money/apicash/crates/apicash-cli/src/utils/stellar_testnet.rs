//! Consulta saldo na Stellar testnet via Horizon HTTP.

use anyhow::{Context, Result};

const DEFAULT_HORIZON: &str = "https://horizon-testnet.stellar.org";

fn horizon_base() -> String {
    std::env::var("APICASH_STELLAR_HORIZON_URL").unwrap_or_else(|_| DEFAULT_HORIZON.into())
}

fn default_account() -> String {
    std::env::var("APICASH_STELLAR_TEST_ACCOUNT")
        .unwrap_or_else(|_| "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF".into())
}

pub async fn print_balance(account_arg: Option<&str>) -> Result<()> {
    let account = match account_arg {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => default_account(),
    };

    let url = format!(
        "{}/accounts/{}",
        horizon_base().trim_end_matches('/'),
        account.trim()
    );

    let res = reqwest::get(&url).await.context("Horizon GET account")?;
    let status = res.status();
    let body = res.text().await?;

    if !status.is_success() {
        println!("HTTP {status}\n{body}");
        return Ok(());
    }

    let v: serde_json::Value = serde_json::from_str(&body).context("parse horizon json")?;
    if let Some(balances) = v.get("balances").and_then(|b| b.as_array()) {
        println!("Conta {} (testnet)\n", account);
        for b in balances {
            if let (Some(asset), Some(amount)) = (b.get("asset"), b.get("balance")) {
                println!("  {asset}: {amount}");
            }
        }
    } else {
        println!("{body}");
    }
    Ok(())
}
