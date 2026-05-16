//! API HTTP **interna** para dashboard administrativo (porta **3001** por padrão).
//!
//! Dependências de domínio: [`CustodyService`], [`DisputeService`], [`AntiFraudeService`], mensageria via [`apicash_events`].

pub mod dto;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod state;

pub use apicash_events as events;

pub use error::{AdminError, ErrorBody};
pub use router::admin_router;
pub use state::AdminState;

pub use dto::{
    AdminDashboardResponse, AdminOrderListResponse, AdminOrderRow, OrderListQuery,
    SellerDashboardResponse, UserScoreListResponse, UserScoreQuery, YieldReportQuery,
    YieldReportResponse,
};

/// Alias semântico para [`admin_router`].
pub fn create_router(state: AdminState) -> axum::Router {
    admin_router(state)
}
