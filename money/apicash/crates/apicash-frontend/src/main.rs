//! Servidor Axum + Leptos SSR (`site-addr` em `Cargo.toml.leptos`).

use std::net::SocketAddr;

use apicash_frontend::App;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use leptos::prelude::*;
use leptos_axum::{file_and_error_handler, generate_route_list, LeptosRoutes};
use leptos_config::get_configuration;
use leptos_meta::*;
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer};


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
    view! {
        <!DOCTYPE html>
        <html lang="pt-BR">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <Title text="HoldFy Admin" />
                <MetaTags />
                <Stylesheet id="apicash" href="/styles/tailwind.css" />
                <AutoReload options=options.clone() />
                <HydrationScripts options=options.clone() />
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

    let site_dir = std::env::var("HOLDFY_SITE_DIST").unwrap_or_else(|_| "/home/jelastic/site-dist".into());
    let admin_dir = std::env::var("HOLDFY_ADMIN_DIST").unwrap_or_else(|_| "/home/jelastic/holdfy-admin-dist".into());

    // Leptos static assets (WASM/JS/CSS) servidos explicitamente
    let pkg_dir = format!("{}/{}", options.site_root, options.site_pkg_dir);
    let styles_dir = format!("{}/styles", options.site_root);

    let options_for_routes = options.clone();
    let gatebox_dir = std::env::var("HOLDFY_GATEBOX_DIST").unwrap_or_else(|_| "/home/jelastic/front-gatebox-dist".into());
    let holdy_dir = std::env::var("HOLDFY_HOLDY_DIST").unwrap_or_else(|_| "/home/jelastic/front-holdy-dist".into());
    let admin_index = format!("{}/index.html", admin_dir);
    let gatebox_index = format!("{}/index.html", gatebox_dir);
    let holdy_index = format!("{}/index.html", holdy_dir);
    let app = Router::new()
        // Proxy para serviços internos bloqueados pelo firewall
        .route("/svc/admin/*path", any(proxy_admin))
        .route("/svc/banco/*path", any(proxy_banco))
        // Leptos assets (WASM, JS, CSS) — antes do fallback SPA
        .nest_service("/pkg", ServeDir::new(pkg_dir))
        .nest_service("/styles", ServeDir::new(styles_dir))
        // Static assets do React site — servidos diretamente (evita wildcard do Leptos)
        .nest_service("/assets", ServeDir::new(format!("{}/assets", site_dir)))
        // holdfy-admin React (dashboard interno) com SPA fallback
        .nest_service("/holdfy-admin",
            ServeDir::new(&admin_dir).not_found_service(ServeFile::new(&admin_index)))
        // front-gatebox React com SPA fallback
        .nest_service("/front-gatebox",
            ServeDir::new(&gatebox_dir).not_found_service(ServeFile::new(&gatebox_index)))
        // front-holdy React (vendedor/comprador) com SPA fallback
        .nest_service("/front-holdy",
            ServeDir::new(&holdy_dir).not_found_service(ServeFile::new(&holdy_index)))
        // Leptos admin SSR — rotas em /admin/*
        .leptos_routes(&state, routes, move || shell(options_for_routes.clone()))
        // React marketplace: SPA fallback via handler customizado
        // (fallback_service não funciona porque leptos_routes adiciona wildcard /*any)
        .fallback(spa_fallback)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!(%addr, "apicash-frontend (Leptos SSR)");


    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve");
}

static HTTP_CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

fn client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().unwrap())
}

async fn proxy_to(base: &str, req: Request) -> Response {
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
    let suffix = path
        .strip_prefix(&format!("/svc/{base}/"))
        .unwrap_or(path.strip_prefix(&format!("/svc/{base}")).unwrap_or(""))
        .to_string();
    let port = if base == "admin" { 3001 } else { 8091 };
    let target = format!("http://127.0.0.1:{port}/{suffix}{query}");

    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes()).unwrap();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), 1 << 20).await.unwrap_or_default();

    let mut rb = client().request(method, &target).body(body);
    for (k, v) in &headers {
        if k != axum::http::header::HOST {
            rb = rb.header(k, v);
        }
    }

    match rb.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut builder = axum::http::Response::builder().status(status);
            for (k, v) in resp.headers() {
                builder = builder.header(k, v);
            }
            let bytes = resp.bytes().await.unwrap_or_default();
            builder.body(Body::from(bytes)).unwrap().into_response()
        }
        Err(_) => (StatusCode::BAD_GATEWAY, "upstream offline").into_response(),
    }
}

async fn proxy_admin(req: Request) -> Response { proxy_to("admin", req).await }
async fn proxy_banco(req: Request) -> Response { proxy_to("banco", req).await }

/// SPA fallback: serve arquivo do site React se existir, senão serve index.html.
/// Necessário porque `leptos_routes` adiciona wildcard `/*any` que impede `fallback_service`.
async fn spa_fallback(req: Request) -> Response {
    let site_dir = std::env::var("HOLDFY_SITE_DIST")
        .unwrap_or_else(|_| "/home/jelastic/site-dist".into());
    let uri_path = req.uri().path();

    // Tenta servir o arquivo estático (assets, favicon, robots.txt, etc.)
    let candidate = format!("{}{}", site_dir, uri_path);
    if let Ok(meta) = tokio::fs::metadata(&candidate).await {
        if meta.is_file() {
            if let Ok(bytes) = tokio::fs::read(&candidate).await {
                let ct = mime_from_path(uri_path);
                return axum::http::Response::builder()
                    .header("content-type", ct)
                    .body(Body::from(bytes))
                    .unwrap()
                    .into_response();
            }
        }
    }

    // SPA fallback: serve index.html para qualquer rota desconhecida
    let index = format!("{}/index.html", site_dir);
    match tokio::fs::read(&index).await {
        Ok(bytes) => axum::http::Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .body(Body::from(bytes))
            .unwrap()
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "site not found").into_response(),
    }
}

fn mime_from_path(path: &str) -> &'static str {
    if path.ends_with(".js") || path.ends_with(".mjs") { "application/javascript" }
    else if path.ends_with(".css") { "text/css" }
    else if path.ends_with(".html") { "text/html; charset=utf-8" }
    else if path.ends_with(".svg") { "image/svg+xml" }
    else if path.ends_with(".png") { "image/png" }
    else if path.ends_with(".ico") { "image/x-icon" }
    else if path.ends_with(".json") { "application/json" }
    else if path.ends_with(".woff2") { "font/woff2" }
    else if path.ends_with(".woff") { "font/woff" }
    else if path.ends_with(".txt") { "text/plain" }
    else { "application/octet-stream" }
}
