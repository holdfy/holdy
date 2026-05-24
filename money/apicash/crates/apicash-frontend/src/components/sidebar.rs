use leptos::prelude::*;
use leptos_router::components::A;

use crate::i18n::{MsgKey, T};

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="ap-sidebar">
            <div class="ap-brand"><T key=MsgKey::Brand /></div>
            <nav>
                <A href="/"><span class="ap-nav-item"><T key=MsgKey::NavDashboard /></span></A>
                <A href="/orders"><span class="ap-nav-item"><T key=MsgKey::NavOrders /></span></A>
                <A href="/disputes"><span class="ap-nav-item"><T key=MsgKey::NavDisputes /></span></A>
                <A href="/sellers"><span class="ap-nav-item"><T key=MsgKey::NavSellers /></span></A>
                <A href="/reports"><span class="ap-nav-item"><T key=MsgKey::NavReports /></span></A>
            </nav>
        </aside>
    }
}
