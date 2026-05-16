use leptos::prelude::*;

use crate::utils::api_client::{get_disputes, resolve_dispute};

#[component]
pub fn DisputeList() -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p class="ap-muted">"A carregar disputas…"</p> }>
            <Await future=get_disputes() let:res>
                {match res {
                    Ok(rows) => view! { <DisputeRows rows=rows.clone() /> }.into_any(),
                    Err(e) => view! { <p class="ap-muted">{format!("{e}")}</p> }.into_any(),
                }}
            </Await>
        </Suspense>
    }
}

#[component]
fn DisputeRows(rows: Vec<serde_json::Value>) -> impl IntoView {
    view! {
        <div class="ap-table-wrap">
            <table class="ap-table">
                <thead>
                    <tr>
                        <th>"ID"</th>
                        <th>"Pedido"</th>
                        <th>"Estado"</th>
                        <th>"Motivo"</th>
                        <th>""</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || rows.clone()
                        key=|v: &serde_json::Value| {
                            v.get("id")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string()
                        }
                        children=|v: serde_json::Value| {
                            let id = v
                                .get("id")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let order = v
                                .get("order_id")
                                .and_then(|x| x.as_str())
                                .unwrap_or("—")
                                .to_string();
                            let status = v
                                .get("status")
                                .and_then(|x| x.as_str())
                                .unwrap_or("—")
                                .to_string();
                            let reason = v
                                .get("reason")
                                .and_then(|x| x.as_str())
                                .unwrap_or("—")
                                .to_string();
                            let id_for_resolve = id.clone();
                            view! {
                                <tr>
                                    <td><code>{id.clone()}</code></td>
                                    <td><code>{order}</code></td>
                                    <td>{status}</td>
                                    <td>{reason}</td>
                                    <td>
                                        <button
                                            type="button"
                                            class="ap-btn ap-btn-primary"
                                            on:click=move |_| {
                                                let id = id_for_resolve.clone();
                                                leptos::task::spawn_local(async move {
                                                    let _ = resolve_dispute(
                                                        id,
                                                        "manual".into(),
                                                        Some("via dashboard".into()),
                                                    )
                                                    .await;
                                                });
                                            }
                                        >
                                            "Resolver (manual)"
                                        </button>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
