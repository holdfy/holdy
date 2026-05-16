// From app/modules/core/messaging/types.go
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Config for simulated gateway failure (testing/failover).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayFailureConfig {
    #[serde(rename = "fail_after_requests")]
    pub fail_after_requests: Option<i32>,
    #[serde(rename = "gateway_recover_after_transactions")]
    pub gateway_recover_after_transactions: Option<i32>,
    #[serde(rename = "current_fallback_transactions")]
    pub current_fallback_transactions: Option<i32>,
    #[serde(rename = "error_code")]
    pub error_code: Option<i32>,
    #[serde(rename = "error_message")]
    pub error_message: Option<String>,
}

/// Payment message published to the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMessage {
    #[serde(rename = "paymentId")]
    pub payment_id: i64,
    pub amount: f64,
    #[serde(rename = "failure_configs", skip_serializing_if = "Option::is_none")]
    pub failure_configs: Option<HashMap<String, GatewayFailureConfig>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_message_roundtrip() {
        let msg = PaymentMessage {
            payment_id: 123,
            amount: 99.99,
            failure_configs: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: PaymentMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.payment_id, 123);
        assert!((parsed.amount - 99.99).abs() < 0.001);
    }

    #[test]
    fn test_payment_message_with_failure_config() {
        let json = r#"{"paymentId":1,"amount":10.5,"failure_configs":{"sulcred":{"fail_after_requests":3}}}"#;
        let msg: PaymentMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.payment_id, 1);
        let cfg = msg.failure_configs.as_ref().unwrap().get("sulcred").unwrap();
        assert_eq!(cfg.fail_after_requests, Some(3));
    }
}
