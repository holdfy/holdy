use leptos::prelude::*;

use crate::utils::api_client::get_seller_dashboard;

#[component]
pub fn SellersPage() -> impl IntoView {
    let seller_id = RwSignal::new(String::new());
    let trigger = RwSignal::new(0u32);

    view! {
        <h1 style="margin-top:0;">"Vendedores"</h1>
        <p class="ap-muted">"UUID do vendedor → GET /admin/sellers/:id/dashboard"</p>
        <div style="display:flex; gap:0.5rem; margin-bottom:1rem; flex-wrap:wrap; align-items:center;">
            <input
                class="ap-input"
                placeholder="UUID do vendedor"
                prop:value=move || seller_id.get()
                on:input=move |ev| seller_id.set(event_target_value(&ev))
            />
            <button
                type="button"
                class="ap-btn ap-btn-primary"
                on:click=move |_| trigger.update(|n| *n += 1)
            >
                "Carregar"
            </button>
        </div>
        <Suspense fallback=|| view! { <p class="ap-muted">"…"</p> }>
            {move || {
                let _ = trigger.get();
                let sid = seller_id.get();
                if sid.len() < 32 {
                    return view! { <p class="ap-muted">"Indique um UUID e clique em Carregar."</p> }.into_any();
                }
                let sid_clone = sid.clone();
                view! {
                    <Await
                        future=get_seller_dashboard(sid_clone)
                        let:res
                    >
                        {match res {
                            Ok(d) => view! {
                                <div class="ap-cards">
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Pedidos"</p>
                                        <p class="ap-card-value">{d.order_count}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Volume"</p>
                                        <p class="ap-card-value">{d.total_volume_minor.clone()}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Score médio"</p>
                                        <p class="ap-card-value">{d.average_risk_score.clone()}</p>
                                    </div>
                                    <div class="ap-card">
                                        <p class="ap-card-title">"Disputas abertas"</p>
                                        <p class="ap-card-value">{d.open_disputes}</p>
                                    </div>
                                </div>
                            }.into_any(),
                            Err(e) => view! { <p class="ap-muted">{format!("{e}")}</p> }.into_any(),
                        }}
                    </Await>
                }
                .into_any()
            }}
        </Suspense>
    }
}
