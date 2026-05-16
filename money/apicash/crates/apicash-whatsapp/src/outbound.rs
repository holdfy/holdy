//! Envio de mensagens — **multi-device** (`whatsapp-rust::Client`) ou Cloud API / stub.

use std::sync::Arc;

use wacore::download::MediaType;
use waproto::whatsapp as wa;
use whatsapp_cloud_api::models::{Interactive, InteractiveActionButton, Message, Text};
use whatsapp_cloud_api::WhatsappClient;

use crate::utils::masking::mask_whatsapp_peer;
use crate::wa_peer::resolve_delivery_jid;

pub enum Outbound {
    /// Transporte alinhado: mensagens via socket Web (sem Graph API).
    Rust {
        client: Arc<whatsapp_rust::Client>,
        sqlite_uri: Option<String>,
    },
    Cloud {
        client: WhatsappClient,
        http: reqwest::Client,
        token: String,
        phone_number_id: String,
    },
    Stub,
}

impl Outbound {
    /// Modo Cloud API (Graph) ou stub — usado quando o transporte **não** é `whatsapp-rust`.
    pub fn from_env() -> Self {
        match (
            std::env::var("WHATSAPP_ACCESS_TOKEN"),
            std::env::var("WHATSAPP_PHONE_NUMBER_ID"),
        ) {
            (Ok(token), Ok(phone_id)) if !token.is_empty() && !phone_id.is_empty() => {
                let client = WhatsappClient::new(&token, &phone_id);
                Outbound::Cloud {
                    client,
                    http: reqwest::Client::new(),
                    token,
                    phone_number_id: phone_id,
                }
            }
            _ => {
                tracing::warn!(
                    "WHATSAPP_ACCESS_TOKEN / WHATSAPP_PHONE_NUMBER_ID ausentes — modo stub"
                );
                Outbound::Stub
            }
        }
    }

    /// Envia texto; em modo Rust devolve `true` se o servidor aceitou o envio.
    pub async fn send_text(&self, to: &str, body: impl AsRef<str>) -> bool {
        let body = body.as_ref();
        match self {
            Outbound::Rust { client, .. } => {
                match resolve_delivery_jid(client.as_ref(), to).await {
                    Ok(jid) => {
                        let m = wa::Message {
                            conversation: Some(body.to_string()),
                            ..Default::default()
                        };
                        match client.send_message(jid.clone(), m).await {
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    peer = %mask_whatsapp_peer(to),
                                    delivery = %jid,
                                    "whatsapp-rust: send_text falhou"
                                );
                                false
                            }
                            Ok(_) => {
                                tracing::info!(
                                    peer = %mask_whatsapp_peer(to),
                                    delivery = %jid,
                                    "whatsapp-rust: send_text ok"
                                );
                                true
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(peer = %mask_whatsapp_peer(to), %e, "whatsapp-rust: jid inválido");
                        false
                    }
                }
            }
            Outbound::Cloud { client, .. } => {
                let msg = Message::from_text(to, Text::new(body), None);
                if let Err(e) = client.send_message(&msg).await {
                    tracing::error!(error = %e, "send_text failed");
                    false
                } else {
                    true
                }
            }
            Outbound::Stub => {
                let to_hint = mask_whatsapp_peer(to);
                tracing::info!(to = %to_hint, %body, "whatsapp stub: text");
                true
            }
        }
    }

    /// Envia PNG como mídia (upload Graph + mensagem imagem). Em stub apenas registra log.
    pub async fn send_image_bytes(&self, to: &str, bytes: &[u8], caption: Option<&str>) {
        match self {
            Outbound::Rust { client, .. } => match resolve_delivery_jid(client.as_ref(), to).await {
                Ok(jid) => {
                    match client.upload(bytes.to_vec(), MediaType::Image).await {
                        Ok(upload) => {
                            let m = wa::Message {
                                image_message: Some(Box::new(wa::message::ImageMessage {
                                    mimetype: Some("image/png".to_string()),
                                    url: Some(upload.url),
                                    direct_path: Some(upload.direct_path),
                                    media_key: Some(upload.media_key),
                                    file_enc_sha256: Some(upload.file_enc_sha256),
                                    file_sha256: Some(upload.file_sha256),
                                    file_length: Some(upload.file_length),
                                    caption: caption.map(|s| s.to_string()),
                                    ..Default::default()
                                })),
                                ..Default::default()
                            };
                            if let Err(e) = client.send_message(jid, m).await {
                                tracing::error!(error = %e, "whatsapp-rust: send_image falhou");
                            }
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "whatsapp-rust: upload QR PNG falhou");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(peer = %mask_whatsapp_peer(to), %e, "whatsapp-rust: jid inválido para imagem")
                }
            },
            Outbound::Cloud {
                http,
                token,
                phone_number_id,
                client,
            } => {
                let url = format!("https://graph.facebook.com/v20.0/{phone_number_id}/media");
                let part = reqwest::multipart::Part::bytes(bytes.to_vec())
                    .file_name("qr.png")
                    .mime_str("image/png")
                    .expect("mime");
                let form = reqwest::multipart::Form::new()
                    .text("messaging_product", "whatsapp")
                    .text("type", "image/png")
                    .part("file", part);

                let upload = match http
                    .post(&url)
                    .bearer_auth(token)
                    .multipart(form)
                    .send()
                    .await
                {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!(error = %e, "media upload failed");
                        return;
                    }
                };

                let js: serde_json::Value = match upload.json().await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::error!(error = %e, "media upload json");
                        return;
                    }
                };

                let Some(media_id) = js["id"].as_str() else {
                    tracing::error!(?js, "media upload missing id");
                    return;
                };

                use whatsapp_cloud_api::models::Image;
                let img = Image::for_id(media_id, caption.map(|s| s.to_string()));
                let msg = Message::from_image(to, img, None);
                if let Err(e) = client.send_message(&msg).await {
                    tracing::error!(error = %e, "send_image failed");
                }
            }
            Outbound::Stub => {
                let to_hint = mask_whatsapp_peer(to);
                tracing::info!(to = %to_hint, len = bytes.len(), ?caption, "whatsapp stub: image");
            }
        }
    }

    /// Menu inicial: novo pedido ou ajuda (Cloud API; stub regista log).
    pub async fn send_welcome_interactive(&self, to: &str, body: impl AsRef<str>) {
        let body = body.as_ref();
        let buttons = vec![
            InteractiveActionButton::new("Novo pedido", "NOVO_PEDIDO"),
            InteractiveActionButton::new("Ajuda", "AJUDA"),
        ];
        let interactive = Interactive::for_button(buttons, body);
        match self {
            Outbound::Rust { .. } => {
                let hint = format!(
                    "{body}\n\n(Responda por texto: *novo pedido* ou *ajuda* — botões Cloud não se aplicam ao multi-device.)"
                );
                self.send_text(to, hint).await;
            }
            Outbound::Cloud { client, .. } => {
                let msg = Message::from_interactive(to, interactive, None);
                if let Err(e) = client.send_message(&msg).await {
                    tracing::error!(error = %e, "welcome interactive failed");
                }
            }
            Outbound::Stub => {
                let to_hint = mask_whatsapp_peer(to);
                tracing::info!(to = %to_hint, %body, "whatsapp stub: welcome interactive");
            }
        }
    }

    /// Botões de confirmação do pedido (antes de criar na API).
    pub async fn send_interactive_confirm_order(&self, to: &str, body: impl AsRef<str>) {
        let body = body.as_ref();
        let buttons = vec![
            InteractiveActionButton::new("Confirmar pedido", "CONFIRMAR_PEDIDO"),
            InteractiveActionButton::new("Cancelar", "CANCELAR"),
        ];
        let interactive = Interactive::for_button(buttons, body);
        match self {
            Outbound::Rust { .. } => {
                let hint = format!("{body}\n\nDigite *CONFIRMAR_PEDIDO* ou *cancelar*.");
                self.send_text(to, hint).await;
            }
            Outbound::Cloud { client, .. } => {
                let msg = Message::from_interactive(to, interactive, None);
                if let Err(e) = client.send_message(&msg).await {
                    tracing::error!(error = %e, "interactive failed");
                }
            }
            Outbound::Stub => {
                let to_hint = mask_whatsapp_peer(to);
                tracing::info!(to = %to_hint, %body, "whatsapp stub: interactive confirm");
            }
        }
    }

    /// Botão após PIX: cliente confirma que pagou (payload `JA_PAGUEI` compatível Cloud).
    pub async fn send_interactive_paid(&self, to: &str, body: impl AsRef<str>) {
        let body = body.as_ref();
        let buttons = vec![InteractiveActionButton::new("Pagamento feito", "JA_PAGUEI")];
        let interactive = Interactive::for_button(buttons, body);
        match self {
            Outbound::Rust { .. } => {
                let hint =
                    format!("{body}\n\nQuando pagar pelo PIX: *pagamento feito*.");
                self.send_text(to, hint).await;
            }
            Outbound::Cloud { client, .. } => {
                let msg = Message::from_interactive(to, interactive, None);
                if let Err(e) = client.send_message(&msg).await {
                    tracing::error!(error = %e, "interactive failed");
                }
            }
            Outbound::Stub => {
                let to_hint = mask_whatsapp_peer(to);
                tracing::info!(to = %to_hint, %body, "whatsapp stub: interactive paid");
            }
        }
    }

    /// Botões de confirmação explícita do recebimento (liberação do escrow).
    pub async fn send_interactive_confirm_receipt(&self, to: &str, body: impl AsRef<str>) {
        let body = body.as_ref();
        let buttons = vec![
            InteractiveActionButton::new("Confirmar recebimento", "CONFIRMAR_RECEBIMENTO"),
            InteractiveActionButton::new("Abrir disputa", "DISPUTA"),
        ];
        let interactive = Interactive::for_button(buttons, body);
        match self {
            Outbound::Rust { .. } => {
                let hint = format!("{body}\n\nDigite *confirmar recebimento* ou *disputa*.");
                self.send_text(to, hint).await;
            }
            Outbound::Cloud { client, .. } => {
                let msg = Message::from_interactive(to, interactive, None);
                if let Err(e) = client.send_message(&msg).await {
                    tracing::error!(error = %e, "interactive failed");
                }
            }
            Outbound::Stub => {
                let to_hint = mask_whatsapp_peer(to);
                tracing::info!(to = %to_hint, %body, "whatsapp stub: interactive confirm_receipt");
            }
        }
    }
}
