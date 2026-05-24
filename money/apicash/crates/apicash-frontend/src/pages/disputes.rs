use leptos::prelude::*;

use crate::components::dispute_list::DisputeList;
use crate::i18n::{MsgKey, T};

#[component]
pub fn DisputesPage() -> impl IntoView {
    view! {
        <h1 style="margin-top:0;"><T key=MsgKey::DisputesTitle /></h1>
        <p class="ap-muted"><T key=MsgKey::DisputesSubtitle /></p>
        <DisputeList />
    }
}
