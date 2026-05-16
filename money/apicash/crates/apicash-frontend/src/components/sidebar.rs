use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="ap-sidebar">
            <div class="ap-brand">"HoldFy"</div>
            <nav>
                <A href="/"><span class="ap-nav-item">"Dashboard"</span></A>
                <A href="/orders"><span class="ap-nav-item">"Pedidos"</span></A>
                <A href="/disputes"><span class="ap-nav-item">"Disputas"</span></A>
                <A href="/sellers"><span class="ap-nav-item">"Vendedores"</span></A>
                <A href="/reports"><span class="ap-nav-item">"Relatórios"</span></A>
            </nav>
        </aside>
    }
}
