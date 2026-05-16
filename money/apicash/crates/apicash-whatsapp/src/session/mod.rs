//! Sessões conversacionais.

mod session_manager;

pub use session_manager::{
    user_id_for_peer_key, CreatingOrderStep, OrderDraft, OrderFlowState, SessionManager,
    UserSession,
};
