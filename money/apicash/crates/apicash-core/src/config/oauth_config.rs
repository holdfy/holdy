//! Configuração dos provedores OAuth (lida de variáveis de ambiente).

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    /// URL base do backend (usada para montar o redirect_uri).
    /// Ex: "http://127.0.0.1:3000" (dev) ou "https://holdfy-dev.sp1.br.saveincloud.net.br/svc/core" (prod)
    pub redirect_base_url: String,
    /// URL do frontend (para redirecionar após callback com o JWT).
    /// Ex: "http://127.0.0.1:5173" (dev) ou "https://holdfy-dev.sp1.br.saveincloud.net.br" (prod)
    pub frontend_url: String,

    // ── Google ──────────────────────────────────────────────────────────────
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,

    // ── Facebook ────────────────────────────────────────────────────────────
    pub facebook_client_id: Option<String>,
    pub facebook_client_secret: Option<String>,

    // ── LinkedIn ────────────────────────────────────────────────────────────
    pub linkedin_client_id: Option<String>,
    pub linkedin_client_secret: Option<String>,

    // ── Apple ───────────────────────────────────────────────────────────────
    pub apple_client_id: Option<String>,
    pub apple_team_id: Option<String>,
    pub apple_key_id: Option<String>,
    pub apple_private_key_p8: Option<String>,
}

impl OAuthConfig {
    pub fn from_env() -> Self {
        Self {
            redirect_base_url: std::env::var("OAUTH_REDIRECT_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".into()),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:5173".into()),
            google_client_id: env_opt("GOOGLE_CLIENT_ID"),
            google_client_secret: env_opt("GOOGLE_CLIENT_SECRET"),
            facebook_client_id: env_opt("FACEBOOK_CLIENT_ID"),
            facebook_client_secret: env_opt("FACEBOOK_CLIENT_SECRET"),
            linkedin_client_id: env_opt("LINKEDIN_CLIENT_ID"),
            linkedin_client_secret: env_opt("LINKEDIN_CLIENT_SECRET"),
            apple_client_id: env_opt("APPLE_CLIENT_ID"),
            apple_team_id: env_opt("APPLE_TEAM_ID"),
            apple_key_id: env_opt("APPLE_KEY_ID"),
            apple_private_key_p8: env_opt("APPLE_PRIVATE_KEY_P8"),
        }
    }

    pub fn google_enabled(&self) -> bool {
        self.google_client_id.is_some() && self.google_client_secret.is_some()
    }

    pub fn facebook_enabled(&self) -> bool {
        self.facebook_client_id.is_some() && self.facebook_client_secret.is_some()
    }

    pub fn linkedin_enabled(&self) -> bool {
        self.linkedin_client_id.is_some() && self.linkedin_client_secret.is_some()
    }

    pub fn apple_enabled(&self) -> bool {
        self.apple_client_id.is_some()
            && self.apple_team_id.is_some()
            && self.apple_key_id.is_some()
            && self.apple_private_key_p8.is_some()
    }

    pub fn redirect_uri(&self, provider: &str) -> String {
        format!("{}/auth/oauth/{provider}/callback", self.redirect_base_url)
    }
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.trim().is_empty())
}
