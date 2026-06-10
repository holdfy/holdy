use leptos::prelude::*;
use leptos_router::components::A;

use crate::i18n::{MsgKey, T};

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="ap-sidebar">
            <div class="ap-brand"><T key=MsgKey::Brand /></div>
            <nav>
                <A href="/admin"><span class="ap-nav-item"><T key=MsgKey::NavDashboard /></span></A>
                <A href="/admin/orders"><span class="ap-nav-item"><T key=MsgKey::NavOrders /></span></A>
                <A href="/admin/disputes"><span class="ap-nav-item"><T key=MsgKey::NavDisputes /></span></A>
                <A href="/admin/sellers"><span class="ap-nav-item"><T key=MsgKey::NavSellers /></span></A>
                <A href="/admin/reports"><span class="ap-nav-item"><T key=MsgKey::NavReports /></span></A>
            </nav>
        </aside>
    }
}
