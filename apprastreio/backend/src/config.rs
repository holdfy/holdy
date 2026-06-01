use std::env;

pub struct Config {
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("LOGISTICA_HTTP_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "8092".to_string());
        Self {
            bind_addr: format!("0.0.0.0:{port}"),
        }
    }
}
