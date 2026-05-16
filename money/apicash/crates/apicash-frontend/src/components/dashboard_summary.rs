use leptos::prelude::*;

use crate::utils::api_client::{get_dashboard_summary, get_user_scores};

/// Cards do dashboard: volume, yield, disputas, score médio, custódias.
#[component]
pub fn DashboardSummaryCards() -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p class="ap-muted">"A carregar…"</p> }>
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
                            view! {
                                <div class="ap-cards">
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Volume total"</p>
                                        <p class="ap-card-value">{data.total_volume_minor.clone()}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Yield acumulado"</p>
                                        <p class="ap-card-value">{data.total_yield_accrued_minor.clone()}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Disputas abertas"</p>
                                        <p class="ap-card-value">{data.open_disputes}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Score médio"</p>
                                        <p class="ap-card-value">
                                            {avg.map(|x| x.to_string()).unwrap_or_else(|| "—".into())}
                                        </p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Custódias travadas"</p>
                                        <p class="ap-card-value">{data.locked_custodies}</p>
                                    </div>
                                </div>
                            }
                            .into_any()
                        }
                        (Err(e), _) | (_, Err(e)) => {
                            view! { <p class="ap-muted">{format!("Erro API: {e}")}</p> }.into_any()
                        }
                    }
                }
            </Await>
        </Suspense>
    }
}
