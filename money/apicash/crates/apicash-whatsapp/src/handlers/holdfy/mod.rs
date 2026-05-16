//! Parser e fluxo de intenção HoldFy (criar pedido / cobrança).

pub mod amount;
pub mod intent;
pub mod parser;
pub mod phone;

pub use amount::extract_amount_from_text;
pub use intent::is_create_holdfy_intent;
pub use parser::{
    next_collect_step, parse_holdfy_message, parse_loose_fields, HoldfyCollectStep,
    ParsedHoldfyMessage,
};
pub use phone::{
    canonical_peer_key, contact_phone_rejected, extract_phone_from_event,
    extract_phone_from_text, normalize_br_mobile, peer_lookup_digit_variants,
};
