//! Cliente OpenAI Vision para análise de evidências de disputa.
//!
//! Env: `OPENAI_API_KEY`, `OPENAI_MODEL` (padrão gpt-4o).

use reqwest::Client;
use serde::Deserialize;

use crate::models::{AiVerdict, EvidenceAnalysisResult};

const DISPUTE_SYSTEM_PROMPT: &str = r#"
Você é um árbitro imparcial de disputas de marketplace. Analise as imagens fornecidas:
- As PRIMEIRAS imagens são do ANÚNCIO ORIGINAL do produto (fotos do vendedor).
- As demais imagens foram ENVIADAS PELO COMPRADOR como prova de problema.

Avalie objetivamente:
1. O produto nas fotos do comprador é o mesmo produto do anúncio? (cor, modelo, marca, detalhes visíveis)
2. O dano ou problema alegado é claramente visível e plausível?
3. Há sinais de manipulação nas fotos do comprador? (imagens genéricas, inconsistência de iluminação, metadados suspeitos, produto diferente)
4. Se há foto de caixa vazia ou danificada: é consistente com o produto esperado?

Responda SOMENTE em JSON sem markdown, no formato:
{"verdict":"favor_buyer"|"favor_seller"|"inconclusive","confidence":0.0-1.0,"reasoning":"...","red_flags":["..."]}
"#;

pub struct OpenAiVisionClient {
    client:    Client,
    api_key:   String,
    model:     String,
}

impl OpenAiVisionClient {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?.trim().to_string();
        if api_key.is_empty() { return None; }
        let model = std::env::var("OPENAI_MODEL")
            .unwrap_or_else(|_| "gpt-4o".to_string());
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .ok()?;
        Some(Self { client, api_key, model })
    }

    /// Envia fotos do anúncio + fotos do comprador ao gpt-4o e retorna o veredito.
    pub async fn analyze(
        &self,
        listing_urls: &[String],
        buyer_urls:   &[String],
    ) -> Result<EvidenceAnalysisResult, String> {
        if listing_urls.is_empty() && buyer_urls.is_empty() {
            return Ok(EvidenceAnalysisResult {
                verdict:    AiVerdict::Inconclusive,
                confidence: 0.0,
                reasoning:  "Nenhuma imagem disponível para análise.".into(),
                red_flags:  vec![],
            });
        }

        let mut content: Vec<serde_json::Value> = vec![
            serde_json::json!({ "type": "text", "text": "Imagens do anúncio original:" }),
        ];
        for url in listing_urls.iter().take(3) {
            content.push(serde_json::json!({
                "type": "image_url",
                "image_url": { "url": url, "detail": "high" }
            }));
        }
        if !buyer_urls.is_empty() {
            content.push(serde_json::json!({
                "type": "text",
                "text": "Evidências enviadas pelo comprador:"
            }));
        }
        for url in buyer_urls.iter().take(5) {
            content.push(serde_json::json!({
                "type": "image_url",
                "image_url": { "url": url, "detail": "high" }
            }));
        }

        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": 800,
            "messages": [
                { "role": "system", "content": DISPUTE_SYSTEM_PROMPT.trim() },
                { "role": "user",   "content": content }
            ]
        });

        let resp = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("openai request: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("openai HTTP {status}: {text}"));
        }

        #[derive(Deserialize)]
        struct Choice { message: Message }
        #[derive(Deserialize)]
        struct Message { content: String }
        #[derive(Deserialize)]
        struct Completion { choices: Vec<Choice> }

        let completion: Completion = resp.json().await
            .map_err(|e| format!("openai deserialize: {e}"))?;

        let raw = completion.choices
            .into_iter().next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        parse_ai_response(&raw)
    }
}

#[derive(Deserialize)]
struct AiResponseRaw {
    verdict:    String,
    confidence: f32,
    reasoning:  String,
    #[serde(default)]
    red_flags:  Vec<String>,
}

fn parse_ai_response(raw: &str) -> Result<EvidenceAnalysisResult, String> {
    // Strip markdown code fence if present.
    let clean = raw.trim()
        .trim_start_matches("```json").trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: AiResponseRaw = serde_json::from_str(clean)
        .map_err(|e| format!("parse AI response: {e} — raw: {clean}"))?;

    let verdict = match parsed.verdict.as_str() {
        "favor_buyer"  => AiVerdict::FavorBuyer,
        "favor_seller" => AiVerdict::FavorSeller,
        _              => AiVerdict::Inconclusive,
    };

    Ok(EvidenceAnalysisResult {
        verdict,
        confidence: parsed.confidence.clamp(0.0, 1.0),
        reasoning:  parsed.reasoning,
        red_flags:  parsed.red_flags,
    })
}
