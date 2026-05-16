//! Configuração de broker, tenant e namespace Pulsar.

use std::env;

/// Configuração carregada de variáveis de ambiente (prefixo `APICASH_PULSAR__` compatível com `apicash-shared`).
#[derive(Debug, Clone)]
pub struct PulsarConfig {
    /// Ex.: `pulsar://127.0.0.1:6650`
    pub service_url: String,
    pub tenant: String,
    pub namespace: String,
    /// Nome do tópico *sem* prefixo persistent:// (montado via [`Self::main_topic`]).
    pub topic_name: String,
}

impl Default for PulsarConfig {
    fn default() -> Self {
        Self {
            service_url: "pulsar://127.0.0.1:6650".into(),
            tenant: "public".into(),
            namespace: "default".into(),
            topic_name: crate::models::MAIN_TOPIC_SUFFIX.into(),
        }
    }
}

impl PulsarConfig {
    /// Lê `APICASH_PULSAR__SERVICE_URL`, `APICASH_PULSAR__TENANT`, `APICASH_PULSAR__NAMESPACE`, `APICASH_PULSAR__TOPIC_NAME`.
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(v) = env::var("APICASH_PULSAR__SERVICE_URL") {
            c.service_url = v;
        }
        if let Ok(v) = env::var("APICASH_PULSAR__TENANT") {
            c.tenant = v;
        }
        if let Ok(v) = env::var("APICASH_PULSAR__NAMESPACE") {
            c.namespace = v;
        }
        if let Ok(v) = env::var("APICASH_PULSAR__TOPIC_NAME") {
            c.topic_name = v;
        }
        c
    }

    /// Tópico persistente completo: `persistent://{tenant}/{namespace}/{topic}`.
    pub fn main_topic(&self) -> String {
        format!(
            "persistent://{}/{}/{}",
            self.tenant, self.namespace, self.topic_name
        )
    }
}
