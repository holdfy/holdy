//! Repositório de usuários persistidos (login social / OAuth).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::Row as _;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OAuthUser {
    pub id: Uuid,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub document: Option<String>,
    pub role: String,
    pub password_hash: Option<String>,
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Busca usuário pelo ID de um provedor OAuth (ex: Google sub).
    async fn find_by_provider(&self, provider: &str, provider_id: &str) -> Option<OAuthUser>;
    /// Busca usuário pelo email.
    async fn find_by_email(&self, email: &str) -> Option<OAuthUser>;
    /// Busca usuário pelo CPF/CNPJ (cadastro self-service com senha).
    async fn find_by_document(&self, document: &str) -> Option<OAuthUser>;
    /// Cria ou atualiza usuário. Retorna o usuário com id preenchido.
    async fn upsert(&self, user: OAuthUser) -> Result<OAuthUser, String>;
    /// Cria usuário novo com CPF/CNPJ + hash de senha (cadastro self-service).
    async fn create_with_password(
        &self,
        document: &str,
        password_hash: &str,
        role: &str,
        name: Option<&str>,
    ) -> Result<OAuthUser, String>;
    /// Vincula um provedor OAuth a um usuário existente.
    async fn link_provider(&self, user_id: Uuid, provider: &str, provider_id: &str) -> Result<(), String>;
    /// Vincula documento (CPF/CNPJ) ao usuário social.
    async fn update_document(&self, user_id: Uuid, document: &str) -> Result<(), String>;
}

// ── In-memory (dev / testes) ──────────────────────────────────────────────────

#[derive(Default)]
pub struct InMemoryUserRepository {
    // user_id → OAuthUser
    users: Arc<RwLock<HashMap<Uuid, OAuthUser>>>,
    // (provider, provider_id) → user_id
    providers: Arc<RwLock<HashMap<(String, String), Uuid>>>,
    // email → user_id
    emails: Arc<RwLock<HashMap<String, Uuid>>>,
    // document → user_id
    documents: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_provider(&self, provider: &str, provider_id: &str) -> Option<OAuthUser> {
        let key = (provider.to_string(), provider_id.to_string());
        let providers = self.providers.read().await;
        let user_id = providers.get(&key)?;
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    async fn find_by_email(&self, email: &str) -> Option<OAuthUser> {
        let emails = self.emails.read().await;
        let user_id = emails.get(email)?;
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    async fn find_by_document(&self, document: &str) -> Option<OAuthUser> {
        let documents = self.documents.read().await;
        let user_id = documents.get(document)?;
        let users = self.users.read().await;
        users.get(user_id).cloned()
    }

    async fn upsert(&self, mut user: OAuthUser) -> Result<OAuthUser, String> {
        if user.id == Uuid::nil() {
            user.id = Uuid::new_v4();
        }
        if let Some(ref email) = user.email {
            let mut emails = self.emails.write().await;
            emails.insert(email.clone(), user.id);
        }
        if let Some(ref document) = user.document {
            let mut documents = self.documents.write().await;
            documents.insert(document.clone(), user.id);
        }
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn create_with_password(
        &self,
        document: &str,
        password_hash: &str,
        role: &str,
        name: Option<&str>,
    ) -> Result<OAuthUser, String> {
        if self.find_by_document(document).await.is_some() {
            return Err("documento já cadastrado".into());
        }
        let user = OAuthUser {
            id: Uuid::new_v4(),
            email: None,
            name: name.map(str::to_string),
            avatar_url: None,
            document: Some(document.to_string()),
            role: role.to_string(),
            password_hash: Some(password_hash.to_string()),
        };
        let mut documents = self.documents.write().await;
        documents.insert(document.to_string(), user.id);
        drop(documents);
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn link_provider(&self, user_id: Uuid, provider: &str, provider_id: &str) -> Result<(), String> {
        let key = (provider.to_string(), provider_id.to_string());
        let mut providers = self.providers.write().await;
        providers.insert(key, user_id);
        Ok(())
    }

    async fn update_document(&self, user_id: Uuid, document: &str) -> Result<(), String> {
        let mut users = self.users.write().await;
        if let Some(u) = users.get_mut(&user_id) {
            u.document = Some(document.to_string());
        }
        Ok(())
    }
}

// ── Postgres ──────────────────────────────────────────────────────────────────

pub struct PostgresUserRepository {
    pool: sqlx::PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

fn row_to_oauth_user(row: sqlx::postgres::PgRow) -> OAuthUser {
    OAuthUser {
        id:            row.get("id"),
        email:         row.get("email"),
        name:          row.get("name"),
        avatar_url:    row.get("avatar_url"),
        document:      row.get("document"),
        role:          row.get("role"),
        password_hash: row.get("password_hash"),
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_provider(&self, provider: &str, provider_id: &str) -> Option<OAuthUser> {
        sqlx::query(
            r#"
            SELECT u.id, u.email, u.name, u.avatar_url, u.document, u.role, u.password_hash
            FROM users u
            JOIN user_social_providers p ON p.user_id = u.id
            WHERE p.provider = $1 AND p.provider_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
        .map(row_to_oauth_user)
    }

    async fn find_by_email(&self, email: &str) -> Option<OAuthUser> {
        sqlx::query(
            "SELECT id, email, name, avatar_url, document, role, password_hash FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
        .map(row_to_oauth_user)
    }

    async fn find_by_document(&self, document: &str) -> Option<OAuthUser> {
        sqlx::query(
            "SELECT id, email, name, avatar_url, document, role, password_hash FROM users WHERE document = $1",
        )
        .bind(document)
        .fetch_optional(&self.pool)
        .await
        .ok()
        .flatten()
        .map(row_to_oauth_user)
    }

    async fn upsert(&self, user: OAuthUser) -> Result<OAuthUser, String> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, name, avatar_url, document, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (email) DO UPDATE
              SET name       = COALESCE(EXCLUDED.name, users.name),
                  avatar_url = COALESCE(EXCLUDED.avatar_url, users.avatar_url)
            RETURNING id, email, name, avatar_url, document, role, password_hash
            "#,
        )
        .bind(user.id)
        .bind(user.email)
        .bind(user.name)
        .bind(user.avatar_url)
        .bind(user.document)
        .bind(user.role)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(row_to_oauth_user(row))
    }

    async fn create_with_password(
        &self,
        document: &str,
        password_hash: &str,
        role: &str,
        name: Option<&str>,
    ) -> Result<OAuthUser, String> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (id, document, password_hash, role, name)
            VALUES (gen_random_uuid(), $1, $2, $3, $4)
            RETURNING id, email, name, avatar_url, document, role, password_hash
            "#,
        )
        .bind(document)
        .bind(password_hash)
        .bind(role)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.as_database_error()
                .is_some_and(|d| d.is_unique_violation())
            {
                "documento já cadastrado".to_string()
            } else {
                e.to_string()
            }
        })?;
        Ok(row_to_oauth_user(row))
    }

    async fn link_provider(&self, user_id: Uuid, provider: &str, provider_id: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO user_social_providers (user_id, provider, provider_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (provider, provider_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .bind(provider_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn update_document(&self, user_id: Uuid, document: &str) -> Result<(), String> {
        sqlx::query("UPDATE users SET document = $1 WHERE id = $2")
            .bind(document)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
