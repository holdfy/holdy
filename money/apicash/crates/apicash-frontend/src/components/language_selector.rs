use leptos::prelude::*;

use crate::i18n::{t, use_i18n, Locale, MsgKey, T};

#[component]
pub fn LanguageSelector() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <label class="ap-lang">
            <span class="ap-muted"><T key=MsgKey::Language /></span>
            <select
                class="ap-lang-select"
                prop:value=move || i18n.locale.get().code()
                on:change=move |ev| {
                    let code = event_target_value(&ev);
                    if let Some(next) = Locale::from_code(&code) {
                        i18n.locale.set(next);
                    }
                }
            >
                {Locale::ALL.into_iter().map(|loc| {
                    let code = loc.code();
                    let label = t(loc, loc.label_key());
                    view! { <option value=code>{label}</option> }
                }).collect_view()}
            </select>
        </label>
    }
}
