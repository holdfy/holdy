use leptos::prelude::*;

use crate::i18n::{MsgKey, T};
use crate::utils::api_client::{get_orders, OrderRow};

#[component]
pub fn OrderTable() -> impl IntoView {
    view! {
        <Suspense fallback=|| view! { <p class="ap-muted"><T key=MsgKey::LoadingOrders /></p> }>
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
                        <th><T key=MsgKey::ColOrder /></th>
                        <th><T key=MsgKey::ColStatus /></th>
                        <th><T key=MsgKey::ColAmount /></th>
                        <th><T key=MsgKey::ColScore /></th>
                        <th><T key=MsgKey::ColDecision /></th>
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
                                    <td><span class="ap-muted"><T key=MsgKey::Detail /></span></td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
