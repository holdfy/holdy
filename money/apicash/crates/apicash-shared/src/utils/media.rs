//! Helpers for building externally-reachable URLs to MinIO-stored media.

use super::x402_policy::public_base_url;

/// Builds the URL clients (browser/WhatsApp) use to fetch a MinIO object.
///
/// By default returns the direct MinIO endpoint URL (works when MinIO is reachable
/// from the client — e.g. local dev with a LAN IP). When `APICASH_MINIO_PROXY=1`
/// (production: MinIO bound to localhost only, unreachable across nodes), returns
/// a proxy URL served by `apicash-core`'s `/media/{bucket}/{key}` route instead,
/// which fetches the object server-side and streams it back.
pub fn minio_object_url(minio_endpoint: &str, bucket: &str, key: &str) -> String {
    let proxied = std::env::var("APICASH_MINIO_PROXY")
        .map(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes"))
        .unwrap_or(false);
    if proxied {
        format!("{}/media/{bucket}/{key}", public_base_url().trim_end_matches('/'))
    } else {
        format!("{}/{bucket}/{key}", minio_endpoint.trim_end_matches('/'))
    }
}
