//! Tabela de transações Stellar com painel de detalhe full-screen.

use leptos::prelude::*;

use crate::i18n::{MsgKey, T};
use crate::utils::api_client::StellarTxRow;

// ── helpers ─────────────────────────────────────────────────────────────────

fn shorten(h: &str) -> String {
    if h.len() > 20 {
        format!("{}…{}", &h[..8], &h[h.len() - 8..])
    } else {
        h.to_string()
    }
}

fn fmt_date(s: &str) -> String {
    // "2025-06-21T12:34:56Z" → "21/06/25 12:34"
    if s.len() < 16 {
        return s.to_string();
    }
    let p: Vec<&str> = s[..10].split('-').collect();
    if p.len() == 3 {
        format!("{}/{}/{} {}", p[2], p[1], &p[0][2..], &s[11..16])
    } else {
        s.to_string()
    }
}

fn fmt_brl(s: &str) -> String {
    format!("R$ {}", s.replace('.', ","))
}

/// Base URL do Stellar Expert para cada rede.
fn explorer_base(network: &str) -> &'static str {
    match network {
        "mainnet" => "https://stellar.expert/explorer/public",
        _         => "https://stellar.expert/explorer/testnet",
    }
}

/// Constrói link para uma TX hash (None se hash começa com "mock" ou "simulated").
fn tx_url(network: &str, hash: &str) -> Option<String> {
    if hash.starts_with("mock") || hash.starts_with("simulated") || hash.is_empty() {
        return None;
    }
    Some(format!("{}/tx/{}", explorer_base(network), hash))
}

/// Constrói link para um contrato Soroban.
fn contract_url_from_network(network: &str, contract_id: &str) -> Option<String> {
    if contract_id.starts_with("mock") || contract_id.is_empty() {
        return None;
    }
    Some(format!("{}/contract/{}", explorer_base(network), contract_id))
}

fn badge_class_mode(m: &str) -> &'static str {
    match m {
        "real"  => "stl-badge stl-badge--real",
        "mock"  => "stl-badge stl-badge--mock",
        _       => "stl-badge stl-badge--sim",
    }
}

fn badge_class_network(n: &str) -> &'static str {
    match n {
        "mainnet" => "stl-badge stl-badge--real",
        "testnet" => "stl-badge stl-badge--mock",
        _         => "stl-badge stl-badge--sim",
    }
}

fn badge_class_status(s: &str) -> &'static str {
    match s {
        "completed"       => "stl-badge stl-badge--real",
        "in_custody"      => "stl-badge stl-badge--custody",
        "pending_funding" => "stl-badge stl-badge--pending",
        _                 => "stl-badge stl-badge--sim",
    }
}

fn explorer_link_label(network: &str) -> String {
    match network {
        "mainnet" => "↗ Stellar Expert (mainnet)".to_string(),
        "testnet" => "↗ Stellar Expert (testnet)".to_string(),
        other     => format!("↗ Stellar Expert ({other})"),
    }
}

// ── public component ─────────────────────────────────────────────────────────

#[component]
pub fn StellarTable(rows: Vec<StellarTxRow>, network: String) -> impl IntoView {
    let selected: RwSignal<Option<StellarTxRow>> = RwSignal::new(None);
    let total = rows.len();

    view! {
        <Show
            when=move || selected.get().is_none()
            fallback=move || {
                match selected.get() {
                    Some(row) => view! { <StellarDetail row=row selected=selected /> }.into_any(),
                    None      => view! {}.into_any(),
                }
            }
        >
            <StellarList rows=rows.clone() total=total network=network.clone() selected=selected />
        </Show>
    }
}

// ── list view ────────────────────────────────────────────────────────────────

#[component]
fn StellarList(
    rows: Vec<StellarTxRow>,
    total: usize,
    network: String,
    selected: RwSignal<Option<StellarTxRow>>,
) -> impl IntoView {
    let net_badge = badge_class_network(&network).to_string();

    view! {
        <div class="stl-list-header">
            <div>
                <p style="margin:0 0 0.2rem;font-weight:600;">
                    {total}" transação(ões) · rede "
                    <span class={net_badge.clone()}>{network.clone()}</span>
                </p>
                <p class="ap-muted" style="margin:0;font-size:0.8rem;">
                    "Clique em uma linha para ver o detalhe completo · "
                    <span class={net_badge}>{network.clone()}</span>
                    " · links abrem o Stellar Expert em nova aba"
                </p>
            </div>
        </div>

        {if rows.is_empty() {
            view! {
                <div class="stl-empty">
                    <T key=MsgKey::StellarNoData />
                </div>
            }.into_any()
        } else {
            view! {
                <div class="ap-table-wrap">
                    <table class="ap-table">
                        <thead>
                            <tr>
                                <th><T key=MsgKey::ColOrder /></th>
                                <th>"Data"</th>
                                <th><T key=MsgKey::StellarBuyer /></th>
                                <th><T key=MsgKey::ColAmount /></th>
                                <th><T key=MsgKey::ColMode /></th>
                                <th><T key=MsgKey::ColNetwork /></th>
                                <th><T key=MsgKey::ColStatus /></th>
                                <th><T key=MsgKey::ColLockTx /></th>
                                // Link direto à rede Stellar
                                <th>"Ver na Rede"</th>
                                <th>""</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || rows.clone()
                                key=|r: &StellarTxRow| r.order_id.clone()
                                children=move |r: StellarTxRow| {
                                    let r2           = r.clone();
                                    let short_id     = r.order_id.chars().take(8).collect::<String>();
                                    let date         = fmt_date(&r.created_at);
                                    let buyer        = if r.buyer_name.is_empty() { "—".to_string() } else { r.buyer_name.clone() };
                                    let amount       = fmt_brl(&r.amount_brl);
                                    let mode_cls     = badge_class_mode(&r.soroban_mode).to_string();
                                    let mode_lbl     = r.soroban_mode.clone();
                                    let net_cls      = badge_class_network(&r.network).to_string();
                                    let net_lbl      = r.network.clone();
                                    let status_cls   = badge_class_status(&r.order_status).to_string();
                                    let status_lbl   = r.order_status.clone();
                                    let lock_short   = r.soroban_lock_tx_hash.as_deref().map(shorten);
                                    // Link para TX lock na rede real — construído aqui para não depender do backend
                                    let stellar_link = r.soroban_lock_tx_hash.as_deref()
                                        .and_then(|h| tx_url(&r.network, h));
                                    let net_for_link = r.network.clone();

                                    view! {
                                        <tr
                                            class="stl-row"
                                            on:click=move |_| selected.set(Some(r2.clone()))
                                        >
                                            <td><code style="font-size:0.8rem;">{short_id}"…"</code></td>
                                            <td class="ap-muted" style="white-space:nowrap;">{date}</td>
                                            <td>{buyer}</td>
                                            <td style="white-space:nowrap;">{amount}</td>
                                            <td><span class={mode_cls}>{mode_lbl}</span></td>
                                            <td><span class={net_cls}>{net_lbl}</span></td>
                                            <td><span class={status_cls}>{status_lbl}</span></td>
                                            <td>
                                                {match lock_short {
                                                    Some(h) => view! {
                                                        <code class="stl-hash">{h}</code>
                                                    }.into_any(),
                                                    None => view! { <span class="ap-muted">"—"</span> }.into_any(),
                                                }}
                                            </td>
                                            // Botão direto para Stellar Expert
                                            <td>
                                                {match stellar_link {
                                                    Some(url) => view! {
                                                        <a
                                                            href={url}
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                            class="stl-net-btn"
                                                        >
                                                            "↗ "
                                                            {match net_for_link.as_str() {
                                                                "mainnet" => "mainnet",
                                                                _         => "testnet",
                                                            }}
                                                        </a>
                                                    }.into_any(),
                                                    None => view! {
                                                        <span class="stl-net-btn--disabled">"—"</span>
                                                    }.into_any(),
                                                }}
                                            </td>
                                            <td class="ap-muted" style="font-size:1rem;">"›"</td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
            }.into_any()
        }}
    }
}

// ── detail view ──────────────────────────────────────────────────────────────

#[component]
fn StellarDetail(row: StellarTxRow, selected: RwSignal<Option<StellarTxRow>>) -> impl IntoView {
    let short_id       = row.order_id.chars().take(8).collect::<String>();
    let full_order_id  = row.order_id.clone();
    let buyer_name     = if row.buyer_name.is_empty() { "—".to_string() } else { row.buyer_name.clone() };
    let buyer_doc      = if row.buyer_document.is_empty() { "—".to_string() } else { row.buyer_document.clone() };
    let seller_id_full = row.seller_id.clone();
    let amount         = fmt_brl(&row.amount_brl);
    let order_status   = row.order_status.clone();
    let custody_status = row.custody_status.clone().unwrap_or_else(|| "—".to_string());
    let created_at     = fmt_date(&row.created_at);

    let network        = row.network.clone();
    let soroban_mode   = row.soroban_mode.clone();

    let contract_id    = row.soroban_escrow_contract_id.clone();
    let lock_hash      = row.soroban_lock_tx_hash.clone();
    let release_hash   = row.soroban_release_tx_hash.clone();
    let brlx_hash      = row.brlx_transfer_tx_hash.clone();

    // Build all explorer URLs locally from network + hash
    let contract_link  = contract_id.as_deref()
        .and_then(|c| contract_url_from_network(&network, c));
    let lock_link      = lock_hash.as_deref()
        .and_then(|h| tx_url(&network, h));
    let release_link   = release_hash.as_deref()
        .and_then(|h| tx_url(&network, h));
    let brlx_link      = brlx_hash.as_deref()
        .and_then(|h| tx_url(&network, h));

    let link_label     = explorer_link_label(&network);
    let link_label2    = link_label.clone();
    let link_label3    = link_label.clone();
    let link_label4    = link_label.clone();

    let status_cls_hdr = badge_class_status(&order_status).to_string();
    let status_cls_row = badge_class_status(&order_status).to_string();
    let mode_cls       = badge_class_mode(&soroban_mode).to_string();
    let net_cls_hdr    = badge_class_network(&network).to_string();
    let net_cls_row    = badge_class_network(&network).to_string();

    view! {
        <div class="stl-detail">
            // ── Header ──────────────────────────────────────────────────────
            <div class="stl-detail-header">
                <button
                    type="button"
                    class="ap-btn"
                    on:click=move |_| selected.set(None)
                >
                    <T key=MsgKey::StellarBack />
                </button>

                <div style="flex:1;min-width:0;">
                    <p style="margin:0;font-size:0.7rem;color:var(--muted);text-transform:uppercase;letter-spacing:0.07em;">
                        "ORDER"
                    </p>
                    <p style="margin:0;font-family:monospace;font-size:1rem;font-weight:700;word-break:break-all;">
                        {short_id}"…"
                    </p>
                </div>

                // Rede Stellar (destaque no header)
                <div class="stl-net-pill">
                    <span class="stl-net-pill-label">"Rede"</span>
                    <span class={net_cls_hdr}>{network.clone()}</span>
                </div>

                <span class={status_cls_hdr}>{order_status.clone()}</span>
                <span style="font-size:1.2rem;font-weight:700;color:var(--success);white-space:nowrap;">
                    {amount.clone()}
                </span>
            </div>

            // ── Two-column grid ──────────────────────────────────────────────
            <div class="stl-detail-grid">

                // ── Left: Negociação ─────────────────────────────────────────
                <div class="stl-section">
                    <p class="stl-section-title"><T key=MsgKey::StellarNegotiation /></p>

                    <DetailRow label_key=MsgKey::ColOrder>
                        <code class="stl-hash stl-hash--full">{full_order_id}</code>
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarBuyer>
                        {buyer_name}
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarDocument>
                        <code class="stl-hash">{buyer_doc}</code>
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarSeller>
                        <code class="stl-hash stl-hash--full">{seller_id_full}</code>
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarAmount>
                        <strong style="color:var(--success);">{amount}</strong>
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarOrderStatus>
                        <span class={status_cls_row}>{order_status}</span>
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarCustodyStatus>
                        {custody_status}
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarCreatedAt>
                        {created_at}
                    </DetailRow>
                </div>

                // ── Right: Blockchain Stellar ─────────────────────────────────
                <div class="stl-section">
                    <p class="stl-section-title"><T key=MsgKey::StellarBlockchain /></p>

                    <DetailRow label_key=MsgKey::StellarNetwork>
                        <span class={net_cls_row}>{network.clone()}</span>
                    </DetailRow>
                    <DetailRow label_key=MsgKey::StellarMode>
                        <span class={mode_cls}>{soroban_mode}</span>
                    </DetailRow>

                    // Contrato Escrow
                    <DetailRow label_key=MsgKey::StellarContract>
                        <HashWithLink hash=contract_id link=contract_link label=link_label empty_msg="—".to_string() />
                    </DetailRow>

                    // TX Lock (PIX → BRLx → Escrow)
                    <DetailRow label_key=MsgKey::StellarLockTx>
                        <HashWithLink hash=lock_hash link=lock_link label=link_label2 empty_msg="— aguardando on-ramp".to_string() />
                    </DetailRow>

                    // TX Release
                    <DetailRow label_key=MsgKey::StellarReleaseTx>
                        <HashWithLink hash=release_hash link=release_link label=link_label3 empty_msg="— aguardando entrega confirmada".to_string() />
                    </DetailRow>

                    // TX BRLx Transfer
                    <DetailRow label_key=MsgKey::StellarBrlxTx>
                        <HashWithLink hash=brlx_hash link=brlx_link label=link_label4 empty_msg="—".to_string() />
                    </DetailRow>
                </div>
            </div>
        </div>
    }
}

// ── sub-components ────────────────────────────────────────────────────────────

/// Hash completo + link para Stellar Expert (ou mensagem de placeholder).
#[component]
fn HashWithLink(
    hash: Option<String>,
    link: Option<String>,
    label: String,
    empty_msg: String,
) -> impl IntoView {
    match hash {
        Some(h) => view! {
            <div>
                <code class="stl-hash stl-hash--full">{h}</code>
                {match link {
                    Some(url) => view! {
                        <div style="margin-top:0.3rem;">
                            <a href={url} target="_blank" rel="noopener noreferrer" class="stl-ext-link">
                                {label}
                            </a>
                        </div>
                    }.into_any(),
                    None => view! {
                        <p class="ap-muted" style="font-size:0.75rem;margin:0.2rem 0 0;">
                            "(hash simulado — sem link on-chain)"
                        </p>
                    }.into_any(),
                }}
            </div>
        }.into_any(),
        None => view! {
            <span class="ap-muted">{empty_msg}</span>
        }.into_any(),
    }
}

/// Linha de detalhe: label à esquerda + conteúdo à direita.
#[component]
fn DetailRow(label_key: MsgKey, children: Children) -> impl IntoView {
    view! {
        <div class="stl-drow">
            <span class="stl-drow-label"><T key=label_key /></span>
            <span class="stl-drow-value">{children()}</span>
        </div>
    }
}
