use std::env;

pub const TOPIC_ANCHOR_REQUESTS: &str = "anchor-requests";

#[derive(Debug, Clone)]
pub struct AnchorConfig {
    pub pulsar_url: String,
    pub pulsar_tenant: String,
    pub pulsar_namespace: String,
    pub topic_full_name: String,
    pub publish_enabled: bool,
}

impl AnchorConfig {
    pub fn from_env() -> Self {
        let url = env::var("PULSAR_URL").unwrap_or_else(|_| "pulsar://localhost:6650".to_string());
        let tenant = env::var("PULSAR_TENANT").unwrap_or_else(|_| "public".to_string());
        let namespace = env::var("PULSAR_NAMESPACE").unwrap_or_else(|_| "default".to_string());
        let topic_full_name = format!("persistent://{}/{}/{}", tenant, namespace, TOPIC_ANCHOR_REQUESTS);
        let publish_enabled = matches!(
            env::var("ANCHOR_PUBLISH_ENABLED").as_deref(),
            Ok("true") | Ok("1")
        );

        Self {
            pulsar_url: url,
            pulsar_tenant: tenant,
            pulsar_namespace: namespace,
            topic_full_name,
            publish_enabled,
        }
    }
}
