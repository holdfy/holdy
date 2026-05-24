use leptos::prelude::*;

use crate::components::language_selector::LanguageSelector;
use crate::i18n::{MsgKey, T};
use crate::providers::auth_provider::auth_user;

#[component]
pub fn Navbar() -> impl IntoView {
    let auth = auth_user();
    let on_logout = move |_| {
        auth.0.set(None);
    };

    view! {
        <header class="ap-nav">
            <span class="ap-muted"><T key=MsgKey::NavbarSubtitle /></span>
            <div style="display:flex; align-items:center; gap:0.75rem;">
                <LanguageSelector />
                {move || auth.0.get().map(|u| view! { <span>{u}</span> })}
                <button type="button" class="ap-btn" on:click=on_logout>
                    <T key=MsgKey::Logout />
                </button>
            </div>
        </header>
    }
}
