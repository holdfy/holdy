//! Ponte HTTP para o backend externo `gatebox/banco/backend_banco` (Banco Saczuck sandbox).
//! Rotas esperadas pelo cliente Go: `POST /api/public/charges/validate` e `POST /api/internal/bank/notify-status`.
//!
//! Validação: prioritiza cobrança na tabela `transaction` (`external_id` ou id numérico); se não achar,
//! aceita QR sintético `GATEBOXRUST:QR:…` (stub PIX) e BR Code EMV `000201…` (tag 54), para o banco
//! poder pagar localmente. `notify-status` marcar transação como concluída (`status_transaction_id=4`)
//! quando `status` for APPROVED/COMPLETED/PAID e `charge_id` for id numérico ou `external_id` conhecido.

mod handler;
mod whatsapp_notify;

pub use handler::{routes, BankBridgeState, QrRefCache};
