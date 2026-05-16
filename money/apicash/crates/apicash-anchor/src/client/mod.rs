//! Separated clients: Anchor (fiat rails) vs Horizon (ledger reads).

mod anchor_client;
mod gatebox_client;
mod stellar_client;

pub use anchor_client::AnchorClient;
pub use gatebox_client::{fetch_dynamic_pix_qrcode, GateboxQrCodeParsed};
pub use stellar_client::StellarClient;
