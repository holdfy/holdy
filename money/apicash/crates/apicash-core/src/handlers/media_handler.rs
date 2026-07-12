//! Proxy para objetos MinIO — usado quando `APICASH_MINIO_PROXY=1` (produção: MinIO
//! só é alcançável de dentro do mesmo nó, nunca exposto direto ao público). O
//! `minio_object_url` armazenado nas evidências/fotos aponta pra cá em vez do
//! endpoint direto do MinIO; este handler busca o objeto (localhost-to-localhost,
//! mesmo host) e devolve os bytes com o content-type original.

use axum::body::Body;
use axum::extract::Path;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;

const ALLOWED_BUCKETS: [&str; 2] = ["holdfy-images", "holdfy-disputes"];

pub async fn proxy_media(Path((bucket, key)): Path<(String, String)>) -> Result<Response, ApiError> {
    if !ALLOWED_BUCKETS.contains(&bucket.as_str()) {
        return Err(ApiError::not_found("unknown bucket"));
    }

    let endpoint = std::env::var("MINIO_ENDPOINT")
        .map_err(|_| ApiError::internal("MINIO_ENDPOINT not configured"))?;
    let url = format!("{}/{}/{}", endpoint.trim_end_matches('/'), bucket, key);

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| ApiError::bad_gateway(format!("media fetch failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(ApiError::not_found("media not found"));
    }

    let content_type = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| header::HeaderValue::from_static("application/octet-stream"));

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| ApiError::internal(format!("media read failed: {e}")))?;

    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("public, max-age=31536000, immutable"),
            ),
        ],
        Body::from(bytes),
    )
        .into_response())
}
