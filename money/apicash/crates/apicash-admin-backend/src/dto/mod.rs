//! DTOs JSON para o painel administrativo.

mod admin_order_response;
mod dashboard;
mod report_request;
mod seller_dashboard;

pub use admin_order_response::{AdminOrderListResponse, AdminOrderRow, OrderListQuery};
pub use dashboard::AdminDashboardResponse;
pub use report_request::{
    UserScoreListResponse, UserScoreQuery, UserScoreRow, YieldReportQuery, YieldReportResponse,
};
pub use seller_dashboard::SellerDashboardResponse;
