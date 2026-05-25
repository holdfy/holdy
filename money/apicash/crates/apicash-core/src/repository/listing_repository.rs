//! Persistência de anúncios importados no PostgreSQL.

use apicash_importer::ProductDraft;
use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
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
}
