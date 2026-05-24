//! Dashboard administrativo HoldFy — **Leptos** com SSR e hydration.
//!
//! - Variáveis: `APICASH_ADMIN_API_URL` (default `http://127.0.0.1:3001`), `APICASH_ADMIN_API_KEY`.
//! - Servidor UI: `Cargo.toml.leptos` → `site-addr` (default `127.0.0.1:3002`).

#![recursion_limit = "512"]

pub mod app;
pub mod components;
pub mod i18n;
pub mod pages;
pub mod providers;
pub mod utils;

pub use app::App;

/// Ponto de entrada WASM para **hydration** (ex.: `cargo leptos build`).
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use leptos::prelude::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(|| view! { <App /> });
}
