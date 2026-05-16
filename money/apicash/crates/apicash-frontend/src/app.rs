//! Raiz da UI: router, layout com sidebar/navbar e páginas.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::pages::{
    dashboard::DashboardPage, disputes::DisputesPage, orders::OrdersPage, reports::ReportsPage,
    sellers::SellersPage,
};
use crate::providers::auth_provider::AuthProvider;

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
        <AuthProvider>
            <Router>
                <Routes fallback=|| {
                    view! { <p class="ap-muted">"Página não encontrada."</p> }
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
    }
}
