//! Provider de idioma com persistência no browser (localStorage).

use leptos::prelude::*;

use crate::i18n::{Locale, I18nContext};

#[cfg(feature = "hydrate")]
const STORAGE_KEY: &str = "holdfy-locale";

#[cfg(feature = "hydrate")]
fn read_stored_locale() -> Option<Locale> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok().flatten()?;
    let code = storage.get_item(STORAGE_KEY).ok().flatten()?;
    Locale::from_code(&code)
}

#[cfg(feature = "hydrate")]
fn persist_locale(locale: Locale) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(STORAGE_KEY, locale.code());
        }
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                let _ = html.set_attribute("lang", locale.code());
            }
        }
    }
}

#[component]
pub fn I18nProvider(children: Children) -> impl IntoView {
    // SSR / generate_route_list: never touch window/localStorage (hydrate feature may still be on).
    let locale = RwSignal::new(Locale::default());
    provide_context(I18nContext { locale });

    #[cfg(feature = "hydrate")]
    {
        Effect::new(move |_| {
            if let Some(stored) = read_stored_locale() {
                locale.set(stored);
            }
        });
        Effect::new(move |_| {
            persist_locale(locale.get());
        });
    }

    children()
}
