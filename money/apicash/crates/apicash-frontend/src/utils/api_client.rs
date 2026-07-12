//! Chamadas ao `apicash-admin-backend` via **server functions** (execução só no servidor SSR).
//!
//! O corpo HTTP (`reqwest`) fica atrás de `feature = "ssr"` para o bundle WASM (hydrate) não
//! puxar futures `!Send` nem depender de `reqwest` no cliente.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
fn admin_base_url() -> String {
    std::env::var("APICASH_ADMIN_API_URL").unwrap_or_else(|_| {
        option_env!("APICASH_ADMIN_API_URL")
            .unwrap_or("http://127.0.0.1:3001")
            .to_string()
    })
}

#[cfg(feature = "ssr")]
fn admin_api_key() -> Result<String, ServerFnError> {
    std::env::var("APICASH_ADMIN_API_KEY")
        .map_err(|_| ServerFnError::new("defina APICASH_ADMIN_API_KEY no servidor"))
}

#[cfg(feature = "ssr")]
async fn admin_get(path: &str) -> Result<reqwest::Response, ServerFnError> {
    let base = admin_base_url();
    let key = admin_api_key()?;
    let url = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    let resp = reqwest::Client::new()
        .get(&url)
        .header("x-api-key", key)
        .send()
        .await?;
    Ok(resp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_volume_minor: String,
    pub total_yield_accrued_minor: String,
    pub open_disputes: usize,
    pub locked_custodies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderRow>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRow {
    pub order_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount_minor: String,
    pub status: String,
    pub risk_score: u32,
    pub risk_decision: String,
    pub custody_id: Option<String>,
    pub created_at: String,
    pub platform_origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldReportResponse {
    pub total_yield_minor: String,
    pub custody_count: usize,
    pub released_count: usize,
    pub period_from: Option<String>,
    pub period_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScoreRow {
    pub user_id: String,
    pub score: u32,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserScoreListResponse {
    pub users: Vec<UserScoreRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerDashboardResponse {
    pub seller_id: String,
    pub order_count: u64,
    pub total_volume_minor: String,
    pub average_risk_score: String,
    pub open_disputes: u64,
}

#[server(GetDashboardSummary, "/api")]
pub async fn get_dashboard_summary() -> Result<DashboardSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let r = admin_get("admin/dashboard").await?;
        let v = r.json::<DashboardSummary>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(GetOrders, "/api")]
pub async fn get_orders() -> Result<OrderListResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let r = admin_get("admin/orders").await?;
        let v = r.json::<OrderListResponse>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(GetDisputes, "/api")]
pub async fn get_disputes() -> Result<Vec<serde_json::Value>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let r = admin_get("admin/disputes").await?;
        let v = r.json::<Vec<serde_json::Value>>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(ResolveDispute, "/api")]
pub async fn resolve_dispute(
    id: String,
    resolution: String,
    notes: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let base = admin_base_url();
        let key = admin_api_key()?;
        let url = format!(
            "{}/admin/disputes/{}/resolve",
            base.trim_end_matches('/'),
            id
        );
        let body = serde_json::json!({
            "resolution": resolution,
            "notes": notes,
        });
        reqwest::Client::new()
            .post(&url)
            .header("x-api-key", key)
            .json(&body)
            .send()
            .await?;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, resolution, notes);
        unreachable!()
    }
}

#[server(GetYieldReport, "/api")]
pub async fn get_yield_report() -> Result<YieldReportResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let r = admin_get("admin/reports/yield").await?;
        let v = r.json::<YieldReportResponse>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(GetUserScores, "/api")]
pub async fn get_user_scores() -> Result<UserScoreListResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let r = admin_get("admin/users/score").await?;
        let v = r.json::<UserScoreListResponse>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StellarTxRow {
    pub order_id: String,
    pub buyer_name: String,
    pub buyer_document: String,
    pub seller_id: String,
    pub amount_brl: String,
    pub order_status: String,
    pub custody_status: Option<String>,
    /// "real" | "mock" | "simulated"
    pub soroban_mode: String,
    pub soroban_escrow_contract_id: Option<String>,
    pub soroban_lock_tx_hash: Option<String>,
    pub soroban_release_tx_hash: Option<String>,
    pub brlx_transfer_tx_hash: Option<String>,
    /// "testnet" | "mainnet" | "simulated"
    pub network: String,
    pub created_at: String,
    pub explorer_lock_url: Option<String>,
    pub explorer_contract_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTxListResponse {
    pub transactions: Vec<StellarTxRow>,
    pub total: usize,
    pub network: String,
}

#[server(GetStellarTransactions, "/api")]
pub async fn get_stellar_transactions() -> Result<StellarTxListResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let r = admin_get("admin/stellar/transactions").await?;
        let v = r.json::<StellarTxListResponse>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!()
    }
}

#[server(GetSellerDashboard, "/api")]
pub async fn get_seller_dashboard(
    seller_id: String,
) -> Result<SellerDashboardResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let base = admin_base_url();
        let key = admin_api_key()?;
        let url = format!(
            "{}/admin/sellers/{}/dashboard",
            base.trim_end_matches('/'),
            seller_id
        );
        let r = reqwest::Client::new()
            .get(&url)
            .header("x-api-key", key)
            .send()
            .await?;
        let v = r.json::<SellerDashboardResponse>().await?;
        Ok(v)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = seller_id;
        unreachable!()
    }
}
