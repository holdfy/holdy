use leptos::prelude::*;

use crate::components::order_table::OrderTable;
use crate::i18n::{MsgKey, T};

#[component]
pub fn OrdersPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;"><T key=MsgKey::OrdersTitle /></h1>
        <p class="ap-muted"><T key=MsgKey::OrdersSubtitle /></p>
        <OrderTable />
    }
}
