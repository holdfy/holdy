use leptos::prelude::*;

use crate::i18n::{t, use_i18n, MsgKey, T};
use crate::utils::api_client::get_yield_report;

/// Barras simples simulando yield por período (placeholder até série temporal real).
#[component]
pub fn YieldChart() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <Suspense fallback=|| view! { <p class="ap-muted"><T key=MsgKey::LoadingYield /></p> }>
            <Await future=get_yield_report() let:res>
                {match res {
                    Ok(r) => {
                        let total = r.total_yield_minor.parse::<f64>().unwrap_or(0.0);
                        let h1 = (total * 0.25).max(4.0);
                        let h2 = (total * 0.35).max(6.0);
                        let h3 = (total * 0.2).max(3.0);
                        let h4 = (total * 0.2).max(3.0);
                        let total_label = t(i18n.locale.get(), MsgKey::YieldTotalReported);
                        let custodies_label = t(i18n.locale.get(), MsgKey::YieldCustodies);
                        view! {
                            <div>
                                <p class="ap-muted">
                                    {total_label} {r.total_yield_minor.clone()}
                                    {custodies_label} {r.custody_count}
                                </p>
                                <div class="ap-chart">
                                    <div class="ap-bar" style=format!("height:{}%", h1.min(100.0))></div>
                                    <div class="ap-bar" style=format!("height:{}%", h2.min(100.0))></div>
                                    <div class="ap-bar" style=format!("height:{}%", h3.min(100.0))></div>
                                    <div class="ap-bar" style=format!("height:{}%", h4.min(100.0))></div>
                                </div>
                            </div>
                        }
                        .into_any()
                    }
                    Err(e) => view! { <p class="ap-muted">{format!("{e}")}</p> }.into_any(),
                }}
            </Await>
        </Suspense>
    }
}
