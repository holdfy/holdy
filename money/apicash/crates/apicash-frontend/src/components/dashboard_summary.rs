use leptos::prelude::*;

use crate::i18n::{t, use_i18n, MsgKey, T};
use crate::utils::api_client::{get_dashboard_summary, get_user_scores};

/// Cards do dashboard: volume, yield, disputas, score médio, custódias.
#[component]
pub fn DashboardSummaryCards() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <Suspense fallback=|| view! { <p class="ap-muted"><T key=MsgKey::Loading /></p> }>
            <Await
                future=async move {
                    let d = get_dashboard_summary().await;
                    let s = get_user_scores().await;
                    (d, s)
                }
                let:pair
            >
                {
                    let (d, s) = pair;
                    match (d, s) {
                        (Ok(data), Ok(scores)) => {
                            let n = scores.users.len();
                            let avg = if n == 0 {
                                None
                            } else {
                                let sum: u32 = scores.users.iter().map(|u| u.score).sum();
                                Some(sum / n as u32)
                            };
                            let dash = t(i18n.locale.get(), MsgKey::Dash);
                            view! {
                                <div class="ap-cards">
                                    <div class="ap-card">
                                        <p class="ap-card-title"><T key=MsgKey::CardTotalVolume /></p>
                                        <p class="ap-card-value">{data.total_volume_minor.clone()}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title"><T key=MsgKey::CardAccumulatedYield /></p>
                                        <p class="ap-card-value">{data.total_yield_accrued_minor.clone()}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title"><T key=MsgKey::CardOpenDisputes /></p>
                                        <p class="ap-card-value">{data.open_disputes}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title"><T key=MsgKey::CardAvgScore /></p>
                                        <p class="ap-card-value">
                                            {avg.map(|x| x.to_string()).unwrap_or_else(|| dash.to_string())}
                                        </p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title"><T key=MsgKey::CardLockedCustodies /></p>
                                        <p class="ap-card-value">{data.locked_custodies}</p>
                                    </div>
                                </div>
                            }
                            .into_any()
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            let prefix = t(i18n.locale.get(), MsgKey::ErrorApi);
                            view! { <p class="ap-muted">{format!("{prefix}{e}")}</p> }.into_any()
                        }
                    }
                }
            </Await>
        </Suspense>
    }
}
