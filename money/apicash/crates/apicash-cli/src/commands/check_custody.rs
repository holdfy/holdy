//! `GET /orders/{id}` — mostra JSON do pedido (inclui dados de custódia quando disponíveis).

use anyhow::{Context, Result};
use uuid::Uuid;

fn base_url() -> String {
    std::env::var("APICASH_HTTP_BASE").unwrap_or_else(|_| "http://127.0.0.1:3000".into())
}

fn bearer() -> Option<String> {
    std::env::var("APICASH_BEARER_TOKEN")
        .ok()
        .filter(|s| !s.is_empty())
}

pub async fn run(order_id: Uuid) -> Result<()> {
    let base = base_url();
    let client = reqwest::Client::new();
    let url = format!("{}/orders/{order_id}", base.trim_end_matches('/'));

    let mut req = client.get(&url);
    if let Some(ref t) = bearer() {
        req = req.header("Authorization", format!("Bearer {t}"));
    }

    let res = req.send().await.context("GET /orders/{id}")?;
    let status = res.status();
    let body = res.text().await?;
    println!("HTTP {status}\n{body}");
    Ok(())
}
