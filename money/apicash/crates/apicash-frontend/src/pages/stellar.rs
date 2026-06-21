use leptos::prelude::*;

use crate::components::stellar_table::StellarTable;
use crate::i18n::{MsgKey, T};
use crate::utils::api_client::get_stellar_transactions;

#[component]
pub fn StellarPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;"><T key=MsgKey::StellarTitle /></h1>
        <p class="ap-muted" style="margin-bottom:1.25rem;"><T key=MsgKey::StellarSubtitle /></p>

        <Suspense fallback=|| view! { <p class="ap-muted"><T key=MsgKey::LoadingStellar /></p> }>
            <Await future=get_stellar_transactions() let:res>
                {match res {
                    Ok(data) => view! {
                        <StellarTable rows=data.transactions.clone() network=data.network.clone() />
                    }.into_any(),
                    Err(e) => view! {
                        <p class="ap-muted">{format!("Erro: {e}")}</p>
                    }.into_any(),
                }}
            </Await>
        </Suspense>
    }
}
