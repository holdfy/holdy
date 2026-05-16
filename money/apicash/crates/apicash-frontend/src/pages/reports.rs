use leptos::prelude::*;

use crate::utils::api_client::{get_user_scores, get_yield_report};

#[component]
pub fn ReportsPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;">"Relatórios"</h1>
        <p class="ap-muted">"Exportação CSV/PDF pode ligar-se a estas server functions."</p>
        <section style="margin-bottom:2rem;">
            <h2>"Yield"</h2>
            <Suspense fallback=|| view! { <p>"…"</p> }>
                <Await future=get_yield_report() let:res>
                    {match res {
                        Ok(r) => {
                            let pretty = serde_json::to_string_pretty(&r).unwrap_or_default();
                            let csv = format!(
                                "total_yield_minor,custody_count,released_count\n{},{},{}",
                                r.total_yield_minor,
                                r.custody_count,
                                r.released_count
                            );
                            view! {
                                <pre style="background:var(--surface); padding:1rem; border-radius:8px; overflow:auto;">
                                    {pretty}
                                </pre>
                                <div style="margin-top:0.5rem;">
                                    <button type="button" class="ap-btn" on:click=move |_| {
                                        let _ = &csv;
                                    }>
                                        "Preparar CSV (copiar no próximo passo)"
                                    </button>
                                </div>
                            }
                            .into_any()
                        }
                        Err(e) => view! { <p>{format!("{e}")}</p> }.into_any(),
                    }}
                </Await>
            </Suspense>
        </section>
        <section>
            <h2>"Scores"</h2>
            <Suspense fallback=|| view! { <p>"…"</p> }>
                <Await future=get_user_scores() let:res>
                    {match res {
                        Ok(list) => view! {
                            <pre style="background:var(--surface); padding:1rem; border-radius:8px; overflow:auto;">
                                {serde_json::to_string_pretty(&list).unwrap_or_default()}
                            </pre>
                        }.into_any(),
                        Err(e) => view! { <p>{format!("{e}")}</p> }.into_any(),
                    }}
                </Await>
            </Suspense>
        </section>
    }
}
