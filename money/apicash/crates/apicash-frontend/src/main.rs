//! Servidor Axum + Leptos SSR (`site-addr` em `Cargo.toml.leptos`).

use std::net::SocketAddr;

use apicash_frontend::App;
use axum::Router;
use leptos::prelude::*;
use leptos_axum::{file_and_error_handler, generate_route_list, LeptosRoutes};
use leptos_config::get_configuration;
use leptos_meta::*;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct LeptosAppState {
    options: LeptosOptions,
}

impl axum::extract::FromRef<LeptosAppState> for LeptosOptions {
    fn from_ref(state: &LeptosAppState) -> Self {
        state.options.clone()
    }
}

fn shell(options: LeptosOptions) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = options;
    view! {
        <!DOCTYPE html>
        <html lang="pt-BR">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <Title text="HoldFy Admin" />
                <MetaTags />
                <Stylesheet id="apicash" href="/styles/tailwind.css" />
                {#[cfg(feature = "hydrate")]
                {
                    view! {
                        <AutoReload options=options.clone() />
                        <HydrationScripts options=options.clone() />
                    }
                }}
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "apicash_frontend=info,tower_http=info".into()),
        )
        .init();

    let toml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml.leptos");
    let conf = if std::path::Path::new(toml_path).exists() {
        get_configuration(Some(toml_path))
    } else {
        get_configuration(None)
    }
    .expect("ler configuração Leptos");
    let mut options = conf.leptos_options;
    // Produção: sobrescreve com LEPTOS_SITE_ADDR se definida (get_configuration(None) usa default :3000)
    if let Ok(env_addr) = std::env::var("LEPTOS_SITE_ADDR") {
        if let Ok(parsed) = env_addr.parse::<SocketAddr>() {
            options.site_addr = parsed;
        }
    }
    let addr: SocketAddr = options.site_addr;

    let state = LeptosAppState {
        options: options.clone(),
    };

    let routes = generate_route_list(App);

    let options_for_routes = options.clone();
    let app = Router::new()
        .leptos_routes(&state, routes, move || shell(options_for_routes.clone()))
        .fallback(file_and_error_handler::<LeptosAppState, _>(shell))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!(%addr, "apicash-frontend (Leptos SSR)");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
}
