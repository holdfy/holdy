//! Estado simples de sessão admin (placeholder até JWT no backend).

use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct AuthContext(pub RwSignal<Option<String>>);

pub fn auth_user() -> AuthContext {
    expect_context::<AuthContext>()
}

#[component]
pub fn AuthProvider(children: Children) -> impl IntoView {
    let user = RwSignal::new(Some("Administrador".to_string()));
    provide_context(AuthContext(user));
    children()
}
