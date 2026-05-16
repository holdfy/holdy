use leptos::prelude::*;

use crate::providers::auth_provider::auth_user;

#[component]
pub fn Navbar() -> impl IntoView {
    let auth = auth_user();
    let on_logout = move |_| {
        auth.0.set(None);
    };

    view! {
        <header class="ap-nav">
            <span class="ap-muted">"Painel interno HoldFy"</span>
            <div style="display:flex; align-items:center; gap:0.75rem;">
                {move || auth.0.get().map(|u| view! { <span>{u}</span> })}
                <button type="button" class="ap-btn" on:click=on_logout>"Sair"</button>
            </div>
        </header>
    }
}
