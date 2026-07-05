//! Arranque do cliente **whatsapp-rust** (multi-device) e encaminhamento para a fila interna.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use image::Luma;
use qrcode::render::unicode::Dense1x2;
use qrcode::{EcLevel, QrCode};
use tokio::sync::mpsc;
use wacore::types::events::Event;
use waproto::whatsapp::device_props::PlatformType;
use whatsapp_rust::bot::Bot;
use whatsapp_rust::pair_code::{PairCodeOptions, PlatformId};
use whatsapp_rust::store::{Backend, SqliteStore};
use whatsapp_rust::TokioRuntime;
use whatsapp_rust_tokio_transport::TokioWebSocketTransportFactory;
use whatsapp_rust_ureq_http_client::UreqHttpClient;

use crate::models::WhatsAppEvent;
use crate::utils::incoming_wa::{parse_incoming_message, IncomingBody};
use crate::wa_peer::{canonical_session_peer_key, peer_key_from_jid};

fn best_qr_for_payload(payload: &[u8]) -> Option<QrCode> {
    [EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H]
        .into_iter()
        .find_map(|ec| QrCode::with_error_correction_level(payload, ec).ok())
}

/// Ficheiro com o QR a **imagem completa** (o terminal muitas vezes part o Unicode por largura).
pub(crate) fn pairing_qr_png_path() -> PathBuf {
    std::env::var("APICASH_WA_QR_PNG")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".runapp/whatsapp-pairing-qr.png"))
}

fn write_pairing_qr_png(qr: &QrCode, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let image = qr
        .render::<Luma<u8>>()
        .module_dimensions(6, 6)
        .quiet_zone(true)
        .build();
    image.save(path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Prévia em texto (muito larga para terminais estreitos); também vai para o log em `INFO`.
/// Nome mostrado em *Aparelhos ligados* (campo `os` do `DeviceProps`; defeito da lib é `"rust"`).
fn linked_device_label() -> String {
    std::env::var("APICASH_WA_DEVICE_LABEL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("APICASH_WA_PUSH_NAME")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| "HoldFy".into())
}

fn render_pairing_qr_dense_preview(payload: &str) -> String {
    let Some(qr) = best_qr_for_payload(payload.as_bytes()) else {
        return "(sem QR: payload inválido)".into();
    };
    qr.render::<Dense1x2>()
        .quiet_zone(true)
        .min_dimensions(1, 1)
        .build()
}

/// Sobe o `Bot`, regista o handler de eventos e devolve o `Client` para envio (`Outbound::Rust`).
///
/// O `run` do bot corre numa task em segundo plano; o erro de arranque reflete falhas em `build`.
pub async fn start_multidevice_bridge(
    tx: mpsc::Sender<WhatsAppEvent>,
    sqlite_uri: &str,
    pair_phone: Option<String>,
    pair_custom_code: Option<String>,
    push_name: Option<String>,
) -> Result<Arc<whatsapp_rust::Client>, Box<dyn std::error::Error + Send + Sync>> {
    let backend: Arc<dyn Backend> = Arc::new(SqliteStore::new(sqlite_uri).await?);

    let transport_factory = TokioWebSocketTransportFactory::new();
    let http_client = UreqHttpClient::new();

    let mut builder = Bot::builder()
        .with_backend(backend)
        .with_transport_factory(transport_factory)
        .with_http_client(http_client)
        .with_runtime(TokioRuntime)
        .skip_history_sync();

    let device_label = linked_device_label();
    let push = push_name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| device_label.clone());
    builder = builder.with_push_name(push);

    // wacore usa os = "rust" por defeito — isso é o que o WhatsApp mostra como nome do aparelho.
    builder = builder.with_device_props(
        Some(device_label.clone()),
        None,
        Some(PlatformType::Chrome),
    );

    if let Some(phone) = pair_phone.filter(|s| !s.is_empty()) {
        builder = builder.with_pair_code(PairCodeOptions {
            phone_number: phone,
            custom_code: pair_custom_code.filter(|s| !s.is_empty()),
            platform_id: PlatformId::Chrome,
            platform_display: format!("{device_label} (Linux)"),
            ..Default::default()
        });
    }

    let wa_sqlite_uri = sqlite_uri.to_string();

    let tx_ev = tx.clone();
    let bot = builder
        .on_event(move |event, _client| {
            let sqlite_for_resolve = wa_sqlite_uri.clone();
            let tx = tx_ev.clone();
            async move {
                match event {
                    Event::PairingQrCode { code, timeout } => {
                        tracing::info!(
                            timeout_secs = timeout.as_secs(),
                            "======== Parear WhatsApp: abra a IMAGEM no ecrã inteiro (o QR no log de texto part-se). WhatsApp > Aparelhos ligados > Ligar um aparelho ========"
                        );
                        let out = pairing_qr_png_path();
                        if let Some(qr) = best_qr_for_payload(code.as_bytes()) {
                            match write_pairing_qr_png(&qr, &out) {
                                Ok(()) => {
                                    let show = std::fs::canonicalize(&out)
                                        .unwrap_or_else(|_| out.clone());
                                    tracing::info!(
                                        path = %show.display(),
                                        "QR completo: abra este ficheiro (foto/visor a ecrã inteiro) e escaneie. Atualiza a cada ~20s — volte a abrir se expirar."
                                    );
                                }
                                Err(e) => tracing::warn!(error = %e, "falha a gravar PNG do QR"),
                            }
                        } else {
                            tracing::error!("pareamento: não foi possível gerar QR; use APICASH_WA_PAIR_PHONE");
                        }
                        let preview = render_pairing_qr_dense_preview(&code);
                        if preview != "(sem QR: payload inválido)" {
                            let log_unicode = std::env::var("APICASH_WA_QR_LOG_UNICODE")
                                .map(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
                                .unwrap_or(true);
                            if log_unicode {
                                tracing::info!(
                                    preview_len = preview.len(),
                                    "Prévia do QR em Unicode (muitas colunas — terminal ou viewer pode partir; o PNG acima é o método fiável):\n{}",
                                    preview
                                );
                            }
                        }
                    }
                    Event::PairingCode { code, timeout } => {
                        tracing::info!(
                            timeout_secs = timeout.as_secs(),
                            "PAIR CODE HoldFy: {code} (WhatsApp > Dispositivos ligados > Ligar com número)"
                        );
                    }
                    Event::Message(msg, info) => {
                        if info.source.is_from_me {
                            return;
                        }
                        if info.source.is_group {
                            tracing::debug!("whatsapp-rust: ignorada mensagem de grupo");
                            return;
                        }
                        let peer = canonical_session_peer_key(
                            peer_key_from_jid(&info.source.sender),
                            Some(sqlite_for_resolve.as_str()),
                        );
                        let mid = info.id.clone();
                        let push_name = {
                            let n = info.push_name.trim().to_string();
                            if n.is_empty() { None } else { Some(n) }
                        };
                        let mut ev = match parse_incoming_message(&msg) {
                            Some(IncomingBody::Text(text)) => WhatsAppEvent::new(peer, mid, text),
                            Some(IncomingBody::ContactPhoneDigits(digits)) => {
                                WhatsAppEvent::with_contact_phone(peer, mid, digits)
                            }
                            Some(IncomingBody::TextAndContact { text, digits }) => {
                                WhatsAppEvent::with_text_and_contact(peer, mid, text, digits)
                            }
                            None => return,
                        };
                        ev.push_name = push_name;
                        if tx.send(ev).await.is_err() {
                            tracing::warn!("whatsapp-rust: fila fechada, evento descartado");
                        }
                    }
                    Event::Connected(_) => {
                        tracing::info!("whatsapp-rust: ligado ao socket");
                    }
                    Event::Disconnected(d) => {
                        tracing::warn!(?d, "whatsapp-rust: desligado");
                    }
                    Event::PairError(e) => {
                        tracing::error!(?e, "whatsapp-rust: erro de pairing");
                    }
                    _ => {}
                }
            }
        })
        .build()
        .await?;

    let client = bot.client();

    tokio::spawn(async move {
        let mut bot = bot;
        match bot.run().await {
            Ok(handle) => {
                if let Err(e) = handle.await {
                    tracing::warn!(error = ?e, "whatsapp-rust: bot terminou");
                }
            }
            Err(e) => tracing::error!(error = %e, "whatsapp-rust: bot.run falhou"),
        }
    });

    Ok(client)
}
