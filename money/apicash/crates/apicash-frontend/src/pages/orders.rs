use leptos::prelude::*;

use crate::components::order_table::OrderTable;

#[component]
pub fn OrdersPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;">"Pedidos"</h1>
        <p class="ap-muted">"Lista com estado, valor e score de risco (origem: admin API)."</p>
        <OrderTable />
    }
}
