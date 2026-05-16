//! Textos padrão em português (BR), curtos para WhatsApp.

pub fn welcome() -> &'static str {
    "Olá, eu sou o *HoldFy*. Protejo seu pagamento com segurança."
}

pub fn menu_hint() -> &'static str {
    "Digite *novo pedido* para começar."
}

pub fn welcome_help() -> &'static str {
    "*Resumo*\n\
     Vendedor manda número do comprador → valor.\n\
     Comprador responde *ACEITO* ou *RECUSO*.\n\
     PIX só depois do aceite; avisamos os dois quando o pagamento for confirmado.\n\n\
     *Comandos:* novo pedido, ajuda, cancelar"
}

pub fn start_order_intro() -> &'static str {
    "Envie o *WhatsApp do comprador* ou um *cartão de contato*; em seguida o *valor*. O PIX só sai depois do comprador responder *ACEITO* aqui."
}

pub fn ask_counterparty() -> &'static str {
    "Informe o *WhatsApp do comprador*:\n\
     número com DDI (ex. *5541999999999*) **ou** *cartão de contato*."
}

pub fn invalid_counterparty_phone() -> &'static str {
    "Número inválido. Envie *10 a 15 dígitos* (com DDI) ou um cartão de contato."
}

pub fn counterparty_same_as_sender() -> &'static str {
    "O comprador tem de ser *outro* WhatsApp (outro número).\n\
     Não dá para cobrar o *mesmo* número que está a falar com o HoldFy.\n\n\
     Envie *cancelar* e recomece com o telefone de quem vai pagar."
}

pub fn ask_amount() -> &'static str {
    "Qual o *valor*? (ex. *150,00*)"
}

/// Proposta ao comprador (B) antes de existir pedido na API.
pub fn buyer_proposal_before_accept(
    seller_phone_masked: &str,
    amount: &str,
    description: &str,
) -> String {
    format!(
        "*Proposta* (vendedor *{seller_phone_masked}*)\n\
         • *R$ {amount}*\n\
         • *{description}*\n\n\
         *ACEITO* = gerar *PIX*. *RECUSO* / *não* / *recuso* = encerrar."
    )
}

/// Vendedor (A): à espera de B.
pub fn seller_waiting_buyer_accept(amount: &str, buyer_phone_masked: &str) -> String {
    format!(
        "Aguardando *ACEITO* de *~{buyer_phone_masked}* (*R$ {amount}*). \
         Você será avisado quando o PIX for criado ou se recusarem."
    )
}

pub fn seller_buyer_refused(amount: &str) -> String {
    format!("Comprador recusou (*R$ {amount}*). *Novo pedido* para tentar de novo.")
}

pub fn seller_proposal_cancelled_by_buyer(amount: &str) -> String {
    format!("Compra cancelada antes do aceite (*R$ {amount}*).")
}

pub fn seller_still_waiting_buyer() -> &'static str {
    "Aguardando o comprador: *ACEITO* ou *RECUSO*."
}

/// Mensagem só com UUID (copiar inteira).
#[must_use]
pub fn order_control_number_only(order_id: &uuid::Uuid) -> String {
    order_id.to_string()
}

/// Vendedor (A): resumo logo após criar pedido; controlo e PIX EMV seguem isolados nas próximas mensagens.
pub fn seller_order_created_notice(
    amount: &str,
    description: &str,
    buyer_phone_masked: &str,
) -> String {
    format!(
        "*Pedido criado*\n\
         • B: ~*{buyer_phone_masked}*\n\
         • *R$ {amount}*\n\
         • *{description}*\n\n\
         A seguir só o código do pedido, depois só o código PIX."
    )
}

/// Comprador (B): antes do controlo / QR / EMV só.
pub fn buyer_payment_intro(amount: &str, description: &str) -> String {
    format!(
        "*Pagamento*\n• *R$ {amount}*\n• *{description}*\n\n\
         Depois vêm só o código do pedido e o código PIX.\n\
         Quando o PIX for pago, avisamos você e o vendedor automaticamente."
    )
}

#[must_use]
pub fn pix_copy_paste_plain(br: &str) -> String {
    br.trim().to_owned()
}

pub fn payment_copy_paste(br: &str) -> String {
    pix_copy_paste_plain(br)
}

/// Aviso automático após confirmação do PIX no Gatebox (vendedor e comprador).
pub fn payment_completed_notify(order_id: &uuid::Uuid, amount: &str) -> String {
    format!(
        "✅ *Pagamento PIX efetuado*\n\
         Pedido `{order_id}` · *R$ {amount}*\n\
         HoldFy confirmou o pagamento desta transação."
    )
}

pub fn awaiting_payment_hint() -> &'static str {
    "Aguardando confirmação automática do PIX pelo Gatebox. Você será avisado aqui quando o pagamento for registrado."
}

pub fn dispute_message() -> &'static str {
    "Disputa registrada. O suporte responde em até 1 dia útil."
}

pub fn cancelled() -> &'static str {
    "Pedido atual cancelado. *Novo pedido* para começar de novo."
}

pub fn invalid_amount() -> &'static str {
    "Valor não entendi. Ex.: *99,90*"
}

pub fn on_ramp_blocked() -> &'static str {
    "Pedido barrado por segurança.\n\nTente depois ou envie dados extras (ex. redes)."
}

pub fn release_ok(amount: &str) -> String {
    format!("Recebimento ok. Liberando *R$ {amount}* ao vendedor (até ~1 min).")
}

pub fn release_unauthorized() -> &'static str {
    "Somente quem pagou pode confirmar o recebimento."
}
