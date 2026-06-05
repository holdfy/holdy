//! Upload de evidências (fotos/vídeos) para MinIO com SHA-256.
//!
//! Variáveis de ambiente: `MINIO_ENDPOINT`, `MINIO_ACCESS_KEY`, `MINIO_SECRET_KEY`.
//! Bucket: `holdfy-disputes` (separado das imagens de produto).

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::DisputeError;

type HmacSha256 = Hmac<Sha256>;

pub struct DisputeImageStore {
    client:     Client,
    endpoint:   String,
    access_key: String,
    secret_key: String,
    bucket:     String,
}

impl DisputeImageStore {
    pub fn from_env() -> Option<Self> {
        let endpoint   = std::env::var("MINIO_ENDPOINT").ok()?.trim_end_matches('/').to_string();
        let access_key = std::env::var("MINIO_ACCESS_KEY").ok()?;
        let secret_key = std::env::var("MINIO_SECRET_KEY").ok()?;
        if endpoint.is_empty() || access_key.is_empty() || secret_key.is_empty() {
            return None;
        }
        let bucket = std::env::var("MINIO_DISPUTES_BUCKET")
            .unwrap_or_else(|_| "holdfy-disputes".to_string());
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .ok()?;
        Some(Self { client, endpoint, access_key, secret_key, bucket })
    }

    /// Faz upload dos bytes para MinIO. Retorna (minio_key, minio_url, sha256_hex).
    pub async fn upload(
        &self,
        dispute_id: Uuid,
        ext: &str,
        bytes: &[u8],
    ) -> Result<(String, String, String), DisputeError> {
        let sha256 = hex_sha256(bytes);
        let key    = format!("disputes/{dispute_id}/{}.{ext}", Uuid::new_v4());
        let mime   = mime_for_ext(ext);
        self.put_object(&key, bytes, mime).await?;
        let url = format!("{}/{}/{}", self.endpoint, self.bucket, key);
        Ok((key, url, sha256))
    }

    async fn put_object(&self, key: &str, body: &[u8], content_type: &str)
        -> Result<(), DisputeError>
    {
        let now          = Utc::now();
        let date_str     = now.format("%Y%m%d").to_string();
        let datetime_str = now.format("%Y%m%dT%H%M%SZ").to_string();
        let region       = "us-east-1";

        let host = self.endpoint
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        let url          = format!("{}/{}/{}", self.endpoint, self.bucket, key);
        let body_hash    = hex_sha256(body);
        let content_len  = body.len().to_string();

        let canonical_headers = format!(
            "content-length:{content_len}\ncontent-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{body_hash}\nx-amz-date:{datetime_str}\n"
        );
        let signed_headers = "content-length;content-type;host;x-amz-content-sha256;x-amz-date";
        let canonical_uri  = format!("/{}/{}", self.bucket, key);
        let canonical_req  = format!(
            "PUT\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{body_hash}"
        );

        let cr_hash          = hex_sha256(canonical_req.as_bytes());
        let credential_scope = format!("{date_str}/{region}/s3/aws4_request");
        let string_to_sign   = format!(
            "AWS4-HMAC-SHA256\n{datetime_str}\n{credential_scope}\n{cr_hash}"
        );
        let signing_key = derive_signing_key(&self.secret_key, &date_str, region, "s3");
        let signature   = hmac_hex(&signing_key, string_to_sign.as_bytes());
        let auth_header = format!(
            "AWS4-HMAC-SHA256 Credential={}/{},SignedHeaders={},Signature={}",
            self.access_key, credential_scope, signed_headers, signature
        );

        let resp = self.client
            .put(&url)
            .header("host", host)
            .header("content-type", content_type)
            .header("content-length", &content_len)
            .header("x-amz-date", &datetime_str)
            .header("x-amz-content-sha256", &body_hash)
            .header("Authorization", &auth_header)
            .body(body.to_vec())
            .send()
            .await
            .map_err(|e| DisputeError::Repository(format!("minio PUT: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            return Err(DisputeError::Repository(format!(
                "minio PUT {key}: HTTP {status} — {body_text}"
            )));
        }
        tracing::info!(key, "disputes: evidence uploaded to MinIO");
        Ok(())
    }
}

pub fn hex_sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn hmac_hex(key: &[u8], data: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key");
    mac.update(data);
    hex::encode(mac.finalize().into_bytes())
}

fn hmac_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac key");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn derive_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let kdate    = hmac_bytes(format!("AWS4{secret}").as_bytes(), date.as_bytes());
    let kregion  = hmac_bytes(&kdate, region.as_bytes());
    let kservice = hmac_bytes(&kregion, service.as_bytes());
    hmac_bytes(&kservice, b"aws4_request")
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "png"  => "image/png",
        "gif"  => "image/gif",
        "webp" => "image/webp",
        "mp4"  => "video/mp4",
        "mov"  => "video/quicktime",
        _      => "image/jpeg",
    }
}
