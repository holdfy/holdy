use leptos::prelude::*;

use crate::utils::api_client::{get_orders, OrderRow};

#[component]
pub fn OrderTable() -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p class="ap-muted">"A carregar pedidos…"</p> }>
            <Await future=get_orders() let:res>
                {match res {
                    Ok(list) => {
                        let rows = list.orders.clone();
                        view! { <OrderTableInner rows=rows /> }.into_any()
                    }
                    Err(e) => view! { <p class="ap-muted">{format!("{e}")}</p> }.into_any(),
                }}
            </Await>
        </Suspense>
    }
}

#[component]
fn OrderTableInner(rows: Vec<OrderRow>) -> impl IntoView {
    view! {
        <div class="ap-table-wrap">
            <table class="ap-table">
                <thead>
                    <tr>
                        <th>"Pedido"</th>
                        <th>"Estado"</th>
                        <th>"Valor"</th>
                        <th>"Score"</th>
                        <th>"Decisão"</th>
                        <th>""</th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || rows.clone()
                        key=|r: &OrderRow| r.order_id.clone()
                        children=|r: OrderRow| {
                            view! {
                                <tr>
                                    <td><code>{r.order_id.clone()}</code></td>
                                    <td>{r.status.clone()}</td>
                                    <td>{r.amount_minor.clone()}</td>
                                    <td>{r.risk_score}</td>
                                    <td>{r.risk_decision.clone()}</td>
                                    <td><span class="ap-muted">"detalhe"</span></td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
