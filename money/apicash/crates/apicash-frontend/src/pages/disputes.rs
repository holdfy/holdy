use leptos::prelude::*;

use crate::components::dispute_list::DisputeList;

#[component]
pub fn DisputesPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;">"Disputas"</h1>
        <p class="ap-muted">"Gestão de disputas; resolução manual preparada."</p>
        <DisputeList />
    }
}
