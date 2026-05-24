//! Estado simples de sessão admin (placeholder até JWT no backend).

use leptos::prelude::*;

use crate::i18n::{t, use_i18n, Locale, MsgKey};

#[derive(Clone, Copy)]
pub struct AuthContext(pub RwSignal<Option<String>>);

pub fn auth_user() -> AuthContext {
    expect_context::<AuthContext>()
}

fn default_admin_name(locale: Locale) -> String {
    t(locale, MsgKey::DefaultAdminUser).to_string()
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let i18n = use_i18n();
    let user = RwSignal::new(Some(default_admin_name(i18n.locale.get())));

    Effect::new(move |_| {
        let locale = i18n.locale.get();
        if user.get().is_some() {
            user.set(Some(default_admin_name(locale)));
        }
    });

    provide_context(AuthContext(user));
    children()
}
