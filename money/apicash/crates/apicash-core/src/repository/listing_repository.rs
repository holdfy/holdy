//! Persistência de anúncios importados no PostgreSQL.

use chrono::{DateTime, Utc};
use apicash_importer::ProductDraft;
use rust_decimal::prelude::ToPrimitive;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct ListingRepository {
    pool: PgPool,
}

impl ListingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Salva um `ProductDraft` e retorna o UUID gerado.
    pub async fn save(
        &self,
        draft: &ProductDraft,
        user_id: Option<Uuid>,
        order_id: Option<Uuid>,
    ) -> Result<Uuid, sqlx::Error> {
        let photos = serde_json::to_value(&draft.photos).unwrap_or_default();
        let platform = format!("{:?}", draft.source_platform).to_ascii_lowercase();

        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO listings (
                user_id, order_id, source_url, source_platform, extractor_used,
                title, description, price_suggested, guarantee, condition,
                location, seller_name, seller_rating, photos, raw_attributes
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15
            ) RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(order_id)
        .bind(draft.source_url.as_str())
        .bind(platform.as_str())
        .bind(draft.extractor_used.as_str())
        .bind(draft.title.as_str())
        .bind(draft.description.as_deref())
        .bind(draft.price_suggested.and_then(|p| p.to_f64()))
        .bind(draft.guarantee.as_deref())
        .bind(draft.condition.as_deref())
        .bind(draft.location.as_deref())
        .bind(draft.seller_name.as_deref())
        .bind(draft.seller_rating.as_deref())
        .bind(photos)
        .bind(&draft.raw_attributes)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.0)
    }

    /// Retorna a chave PIX registrada no wa_contacts para um user_id.
    /// Usada pelo off-ramp de disputa para saber para onde enviar o PIX.
    pub async fn pix_key_for_user(&self, user_id: Uuid) -> Option<String> {
        let row = sqlx::query(
            "SELECT pix_key FROM wa_contacts WHERE user_id = $1 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None)?;

        row.try_get::<Option<String>, _>("pix_key").ok().flatten()
    }

    /// Retorna as URLs de fotos do anúncio vinculado a um pedido (para análise IA de disputa).
    pub async fn photos_for_order(&self, order_id: Uuid) -> Vec<String> {
        let row = sqlx::query("SELECT photos FROM listings WHERE order_id = $1 LIMIT 1")
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);

        let Some(r) = row else { return vec![]; };
        let val: serde_json::Value = r.try_get("photos").unwrap_or(serde_json::Value::Null);
        match val {
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .take(3)
                .collect(),
            _ => vec![],
        }
    }

    /// Vincula um listing a um pedido existente.
    pub async fn set_order_id(&self, listing_id: Uuid, order_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE listings SET order_id = $1 WHERE id = $2")
            .bind(order_id)
            .bind(listing_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// ─── ImportJob ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ImportJob {
    pub id: Uuid,
    pub url: String,
    pub user_id: Option<Uuid>,
    pub status: String,
    pub listing_id: Option<Uuid>,
    pub error_msg: Option<String>,
    pub queued_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl ListingRepository {
    /// Cria um job com status `queued` e retorna o UUID gerado.
    pub async fn create_import_job(
        &self,
        url: &str,
        user_id: Option<Uuid>,
    ) -> Result<Uuid, sqlx::Error> {
        let row: (Uuid,) = sqlx::query_as(
            "INSERT INTO import_jobs (url, user_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(url)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    /// Marca job como `done` após scraping bem-sucedido.
    pub async fn complete_import_job(
        &self,
        job_id: Uuid,
        listing_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE import_jobs
            SET status = 'done', listing_id = $1, completed_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(listing_id)
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Marca job como `error` com mensagem.
    pub async fn fail_import_job(
        &self,
        job_id: Uuid,
        error_msg: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE import_jobs
            SET status = 'error', error_msg = $1, completed_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(error_msg)
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Lê status do job por UUID.
    pub async fn get_import_job(&self, job_id: Uuid) -> Result<Option<ImportJob>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, url, user_id, status, listing_id, error_msg, queued_at, completed_at
            FROM import_jobs WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ImportJob {
            id: r.try_get("id").unwrap(),
            url: r.try_get("url").unwrap(),
            user_id: r.try_get("user_id").unwrap_or(None),
            status: r.try_get("status").unwrap(),
            listing_id: r.try_get("listing_id").unwrap_or(None),
            error_msg: r.try_get("error_msg").unwrap_or(None),
            queued_at: r.try_get("queued_at").unwrap(),
            completed_at: r.try_get("completed_at").unwrap_or(None),
        }))
    }
}
