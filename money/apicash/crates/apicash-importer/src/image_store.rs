//! MinIO image store: download external image URL → re-upload to MinIO → return public URL.
//!
//! Auth: AWS SigV4 (minimal implementation for single PUT request).
//! Env vars: `MINIO_ENDPOINT`, `MINIO_ACCESS_KEY`, `MINIO_SECRET_KEY`, `MINIO_BUCKET`.
//! Public URL pattern: `{endpoint}/{bucket}/{key}`.

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::ImporterError;

type HmacSha256 = Hmac<Sha256>;

pub struct MinioImageStore {
    client: Client,
    endpoint: String,
    access_key: String,
    secret_key: String,
    bucket: String,
}

impl MinioImageStore {
    /// Build from env vars. Returns `None` if any required var is missing.
    pub fn from_env() -> Option<Self> {
        let endpoint = std::env::var("MINIO_ENDPOINT").ok()?.trim_end_matches('/').to_string();
        let access_key = std::env::var("MINIO_ACCESS_KEY").ok()?;
        let secret_key = std::env::var("MINIO_SECRET_KEY").ok()?;
        let bucket = std::env::var("MINIO_BUCKET")
            .unwrap_or_else(|_| "holdfy-images".to_string());

        if endpoint.is_empty() || access_key.is_empty() || secret_key.is_empty() {
            return None;
        }

        let client = Client::builder()
            .user_agent("HoldfyImporter/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok()?;

        Some(Self { client, endpoint, access_key, secret_key, bucket })
    }

    /// Download `external_url`, upload to MinIO, return public URL.
    pub async fn upload_from_url(&self, external_url: &str) -> Result<String, ImporterError> {
        let bytes = self
            .client
            .get(external_url)
            .send()
            .await
            .map_err(|e| ImporterError::FetchFailed(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| ImporterError::FetchFailed(e.to_string()))?;

        let ext = external_url
            .split('?').next().unwrap_or("")
            .rsplit('.').next().filter(|e| e.len() <= 5)
            .unwrap_or("jpg");
        let key = format!("products/{}.{}", Uuid::new_v4(), ext);
        let content_type = mime_for_ext(ext);

        self.put_object(&key, bytes.as_ref(), content_type).await?;

        Ok(apicash_shared::minio_object_url(&self.endpoint, &self.bucket, &key))
    }

    async fn put_object(
        &self,
        key: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<(), ImporterError> {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let datetime_str = now.format("%Y%m%dT%H%M%SZ").to_string();

        let region = "us-east-1"; // MinIO ignores region but SigV4 needs one

        // Extract host from endpoint
        let host = self.endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://");

        let url = format!("{}/{}/{}", self.endpoint, self.bucket, key);

        let body_hash = hex_sha256(body);
        let content_length = body.len().to_string();

        // Canonical headers (sorted alphabetically by name)
        let canonical_headers = format!(
            "content-length:{content_length}\ncontent-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{body_hash}\nx-amz-date:{datetime_str}\n"
        );
        let signed_headers = "content-length;content-type;host;x-amz-content-sha256;x-amz-date";
        let canonical_uri = format!("/{}/{}", self.bucket, key);
        let canonical_request = format!(
            "PUT\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{body_hash}"
        );

        let cr_hash = hex_sha256(canonical_request.as_bytes());
        let credential_scope = format!("{date_str}/{region}/s3/aws4_request");
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{datetime_str}\n{credential_scope}\n{cr_hash}"
        );

        let signing_key = derive_signing_key(&self.secret_key, &date_str, region, "s3");
        let signature = hmac_hex(&signing_key, string_to_sign.as_bytes());

        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{},SignedHeaders={},Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        let resp = self
            .client
            .put(&url)
            .header("host", host)
            .header("content-type", content_type)
            .header("content-length", &content_length)
            .header("x-amz-date", &datetime_str)
            .header("x-amz-content-sha256", &body_hash)
            .header("Authorization", &auth_header)
            .body(body.to_vec())
            .send()
            .await
            .map_err(|e| ImporterError::FetchFailed(format!("minio PUT: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(ImporterError::FetchFailed(format!(
                "minio PUT {key}: HTTP {status} — {body_text}"
            )));
        }

        tracing::info!(key, "minio: image uploaded");
        Ok(())
    }
}

fn hex_sha256(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hex::encode(hash)
}

fn hmac_hex(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

fn hmac_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn derive_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let key_date = hmac_bytes(format!("AWS4{secret}").as_bytes(), date.as_bytes());
    let key_region = hmac_bytes(&key_date, region.as_bytes());
    let key_service = hmac_bytes(&key_region, service.as_bytes());
    hmac_bytes(&key_service, b"aws4_request")
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "image/jpeg",
    }
}
