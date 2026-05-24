use leptos::prelude::*;

use crate::components::{dashboard_summary::DashboardSummaryCards, yield_chart::YieldChart};
use crate::i18n::{MsgKey, T};

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;"><T key=MsgKey::DashboardTitle /></h1>
        <p class="ap-muted"><T key=MsgKey::DashboardSubtitle /></p>
        <DashboardSummaryCards />
        <h2><T key=MsgKey::DashboardYieldSection /></h2>
        <YieldChart />
    }
}
