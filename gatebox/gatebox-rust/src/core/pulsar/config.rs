// From app/modules/core/pulsar/config.go
use std::time::Duration;

pub const TOPIC_PAYMENT: &str = "payment-queue";
pub const TOPIC_RETRY: &str = "payment-queue-dlx";
pub const TOPIC_HOSPITAL: &str = "payment-queue-hospital";
pub const SUBSCRIPTION_NAME: &str = "payment-sub";
pub const MAX_RETRIES: u32 = 3;
pub const RETRY_DELAY_SECS: u64 = 15;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub topic_full_name: String,
    pub topic_hospital_full: String,
    pub subscription_name: String,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for Config {
    fn default() -> Self {
        let url = std::env::var("PULSAR_URL").unwrap_or_else(|_| "pulsar://localhost:6650".to_string());
        let tenant = std::env::var("PULSAR_TENANT").unwrap_or_else(|_| "public".to_string());
        let namespace = std::env::var("PULSAR_NAMESPACE").unwrap_or_else(|_| "default".to_string());
        let topic_full_name = format!("persistent://{}/{}/{}", tenant, namespace, TOPIC_PAYMENT);
        let topic_hospital_full = format!("persistent://{}/{}/{}", tenant, namespace, TOPIC_HOSPITAL);
        Config {
            url,
            topic_full_name,
            topic_hospital_full,
            subscription_name: SUBSCRIPTION_NAME.to_string(),
            max_retries: MAX_RETRIES,
            retry_delay: Duration::from_secs(RETRY_DELAY_SECS),
        }
    }
}
