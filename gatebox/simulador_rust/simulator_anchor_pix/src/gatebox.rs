//! Cliente opcional para obter QR EMV real do Gatebox/Sulcred.
//! Quando GATEBOX_BASE_URL está configurado, o simulador obtém um QR real em vez de gerar um fake.

use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GbQrResponse {
    qr_code: Option<String>,
    #[serde(default)]
    status_code: i32,
}

/// Chama `POST {gatebox_base}/api/v1/pix/qrcode` e retorna o payload EMV.
/// Retorna `None` se o Gatebox não estiver disponível — caller usa QR fake.
pub async fn fetch_qr_from_gatebox(
    http: &Client,
    gatebox_base: &str,
    api_key: Option<&str>,
    amount: &str,
    memo: &str,
) -> Option<String> {
    let url = format!("{}/api/v1/pix/qrcode", gatebox_base.trim_end_matches('/'));
    let payer_name = std::env::var("GATEBOX_CUSTOMER_NAME")
        .unwrap_or_else(|_| "HoldFy Testnet".into());
    let payer_doc = std::env::var("GATEBOX_CUSTOMER_DOCUMENT").unwrap_or_default();

    let body = serde_json::json!({
        "amount": amount.parse::<f64>().unwrap_or(0.0),
        "payerName": payer_name,
        "payerDocument": payer_doc,
        "description": format!("HoldFy anchor-sim {memo}"),
        "expirationSeconds": 1800,
        "reference": memo,
    });

    let mut req = http.post(&url).json(&body);
    if let Some(key) = api_key {
        req = req.bearer_auth(key);
    }

    match req.send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<GbQrResponse>().await {
                Ok(r) if r.qr_code.as_deref().map(|s| !s.is_empty()).unwrap_or(false) => {
                    r.qr_code
                }
                Ok(_) => {
                    warn!("gatebox: qr_code ausente na resposta");
                    None
                }
                Err(e) => {
                    warn!(error = %e, "gatebox: parse da resposta falhou");
                    None
                }
            }
        }
        Ok(resp) => {
            warn!(status = %resp.status(), "gatebox: resposta não-2xx");
            None
        }
        Err(e) => {
            warn!(error = %e, "gatebox: erro de rede");
            None
        }
    }
}

/// Gera um QR EMV fake mas plausível quando o Gatebox não está disponível.
/// O campo `additionalData` inclui o memo para rastreabilidade.
pub fn fake_pix_qr(amount: &str, memo: &str) -> String {
    // Estrutura mínima de um PIX copia-e-cola (EMV simplificado)
    // Não é um QR válido para pagamento real — apenas para testes de integração
    let amount_field = if amount.is_empty() || amount == "0" {
        String::new()
    } else {
        // Tag 54 = transaction amount
        let val = amount;
        format!("54{:02}{}", val.len(), val)
    };

    let memo_truncated = &memo[..memo.len().min(25)];
    let additional = format!("0525{:025}", memo_truncated);

    // Mínimo EMV: payload format (00), point of initiation (01), merchant account (26), currency (53), additional (62)
    format!(
        "000201\
         010212\
         26360014br.gov.bcb.pix0114sim_anchor_test\
         52040000\
         5303986\
         {}\
         5802BR\
         5913HoldFy Testnet\
         6006Brasil\
         62{:02}{}\
         6304ABCD",
        amount_field,
        additional.len(),
        additional
    )
}
