use leptos::prelude::*;
use leptos_router::components::A;

use crate::i18n::{MsgKey, T};

#[component]
pub fn Sidebar() -> impl IntoView {
    // Só afeta o texto do `href` devolvido ao navegador (nginx remove `/admin/` antes de
    // repassar ao backend, então as rotas em si continuam sem prefixo — ver app.rs).
    let base = std::env::var("APICASH_FRONTEND_BASE_PATH").unwrap_or_default();
    let link = |path: &str| format!("{base}{path}");

    view! {
        <aside class="ap-sidebar">
            <div class="ap-brand"><T key=MsgKey::Brand /></div>
            <nav>
                <A href=link("/")><span class="ap-nav-item"><T key=MsgKey::NavDashboard /></span></A>
                <A href=link("/orders")><span class="ap-nav-item"><T key=MsgKey::NavOrders /></span></A>
                <A href=link("/disputes")><span class="ap-nav-item"><T key=MsgKey::NavDisputes /></span></A>
                <A href=link("/sellers")><span class="ap-nav-item"><T key=MsgKey::NavSellers /></span></A>
                <A href=link("/reports")><span class="ap-nav-item"><T key=MsgKey::NavReports /></span></A>
                <A href=link("/stellar")><span class="ap-nav-item"><T key=MsgKey::NavStellar /></span></A>
            </nav>
        </aside>
    }
}
