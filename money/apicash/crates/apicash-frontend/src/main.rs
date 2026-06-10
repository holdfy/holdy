//! Servidor Axum + Leptos SSR (`site-addr` em `Cargo.toml.leptos`).

use std::net::SocketAddr;

use apicash_frontend::App;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
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

    // Proxy reverso para serviços internos (acessíveis via porta 3002 pelo firewall Jelastic).
    // Usa nest() + fallback() para evitar rotas catch-all que causam pânico no matchit 0.7.
    // O nest() já strip o prefixo antes de entregar ao handler, então proxy_upstream não precisa stripar.
    let admin_router: Router<LeptosAppState> =
        Router::new().fallback(axum::routing::any(proxy_admin));
    let banco_router: Router<LeptosAppState> =
        Router::new().fallback(axum::routing::any(proxy_banco));
    let tracking_router: Router<LeptosAppState> =
        Router::new().fallback(axum::routing::any(proxy_tracking));

    let app = Router::new()
        .nest("/svc/admin", admin_router)
        .nest("/svc/banco", banco_router)
        .nest("/svc/tracking", tracking_router)
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

// Proxy reverso: /svc/admin/* → http://127.0.0.1:3001/*
// nest() já stripou o prefixo "/svc/admin" antes de chegar aqui.
async fn proxy_admin(req: Request) -> Response {
    proxy_to(req, "http://127.0.0.1:3001").await
}

// Proxy reverso: /svc/banco/* → http://127.0.0.1:8091/*
// nest() já stripou o prefixo "/svc/banco" antes de chegar aqui.
async fn proxy_banco(req: Request) -> Response {
    proxy_to(req, "http://127.0.0.1:8091").await
}

// Proxy reverso: /svc/tracking/* → http://127.0.0.1:8092/*
// nest() já stripou o prefixo "/svc/tracking" antes de chegar aqui.
async fn proxy_tracking(req: Request) -> Response {
    proxy_to(req, "http://127.0.0.1:8092").await
}

async fn proxy_to(req: Request, upstream: &str) -> Response {
    let (parts, body) = req.into_parts();

    // nest() já entrega o path sem o prefixo /svc/<svc> — usa diretamente.
    let path_and_query = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let bare = if path_and_query.is_empty() { "/" } else { path_and_query };

    let target_uri: Uri = match format!("{upstream}{bare}").parse::<Uri>() {
        Ok(u) => u,
        Err(_) => {
            return (StatusCode::BAD_GATEWAY, "proxy uri error").into_response();
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();

    let body_bytes = match axum::body::to_bytes(body, 10 * 1024 * 1024).await {
        Ok(b) => b,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "failed to read body").into_response();
        }
    };

    let mut req_builder = client
        .request(parts.method.clone(), target_uri.to_string())
        .body(body_bytes);

    // Forward headers (skip host and content-length — reqwest sets them)
    for (name, value) in &parts.headers {
        let n = name.as_str();
        if n == "host" || n == "content-length" {
            continue;
        }
        req_builder = req_builder.header(name.clone(), value.clone());
    }

    match req_builder.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let mut builder = axum::response::Response::builder().status(status);
            for (name, value) in resp.headers() {
                if name.as_str() != "transfer-encoding" {
                    builder = builder.header(name, value);
                }
            }
            let bytes = resp.bytes().await.unwrap_or_default();
            builder.body(Body::from(bytes)).unwrap_or_else(|_| {
                (StatusCode::BAD_GATEWAY, "proxy response error").into_response()
            })
        }
        Err(e) => {
            tracing::warn!(target=%target_uri, err=%e, "proxy upstream error");
            (StatusCode::BAD_GATEWAY, format!("upstream error: {e}")).into_response()
        }
    }
}
