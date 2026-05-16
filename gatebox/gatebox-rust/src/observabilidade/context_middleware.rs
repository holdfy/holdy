// Converted from gateboxgo/utils/observabilidade/context_middleware.go
use axum::{extract::Request, middleware::Next, response::Response};
use rand::Rng;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn generate_request_id() -> String {
    let bytes: [u8; 8] = rand::thread_rng().gen();
    hex::encode(bytes)
}

/// Extension key for request ID (axum uses extensions).
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .or_else(|| request.headers().get("X-Request-ID"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(generate_request_id);

    let mut response = next.run(request).await;

    if let (Ok(name), Ok(val)) = (
        axum::http::header::HeaderName::try_from("x-request-id"),
        axum::http::header::HeaderValue::try_from(request_id),
    ) {
        response.headers_mut().insert(name, val);
    }
    response
}

pub fn get_request_id_from_extension(extensions: &axum::http::Extensions) -> String {
    extensions
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| "unknown".to_string())
}
