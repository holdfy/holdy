use leptos::prelude::*;

use crate::components::{dashboard_summary::DashboardSummaryCards, yield_chart::YieldChart};

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;">"Dashboard"</h1>
        <p class="ap-muted">"Resumo operacional e indicadores de risco."</p>
        <DashboardSummaryCards />
        <h2>"Yield (visualização)"</h2>
        <YieldChart />
    }
}
