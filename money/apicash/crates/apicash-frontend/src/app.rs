//! Raiz da UI: router, layout com sidebar/navbar e páginas.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::i18n::{MsgKey, T};
use crate::pages::{
    dashboard::DashboardPage, disputes::DisputesPage, orders::OrdersPage, reports::ReportsPage,
    sellers::SellersPage,
};
use crate::providers::auth_provider::AuthProvider;
use crate::providers::i18n_provider::I18nProvider;

#[component]
fn Layout() -> impl IntoView {
    view! {
        <div class="ap-layout">
            <crate::components::sidebar::Sidebar />
            <div class="ap-right">
                <crate::components::navbar::Navbar />
                <main class="ap-main">
                    <Outlet />
                </main>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <I18nProvider>
            <AuthProvider>
                <Router>
                    <Routes fallback=|| {
                        view! { <p class="ap-muted"><T key=MsgKey::PageNotFound /></p> }
                    }>
                        <ParentRoute path=path!("") view=Layout>
                            <Route path=path!("") view=DashboardPage />
                            <Route path=path!("orders") view=OrdersPage />
                            <Route path=path!("disputes") view=DisputesPage />
                            <Route path=path!("sellers") view=SellersPage />
                            <Route path=path!("reports") view=ReportsPage />
                        </ParentRoute>
                    </Routes>
                </Router>
            </AuthProvider>
        </I18nProvider>
    }
}
