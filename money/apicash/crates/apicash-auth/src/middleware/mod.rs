//! Middleware Axum (`verify_core_gateway`, `verify_admin`).

mod jwt_middleware;

pub use jwt_middleware::{
    claims_from_extensions, get_current_user_id, verify_admin, verify_core_gateway,
};
