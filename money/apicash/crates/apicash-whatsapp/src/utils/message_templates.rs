//! Textos padrão em português (BR), curtos para WhatsApp.

/// Formata CPF (`123.456.789-01`) ou CNPJ (`12.345.678/0001-95`) para exibição.
pub fn format_document(doc: &str) -> String {
    let d: String = doc.chars().filter(|c| c.is_ascii_digit()).collect();
    match d.len() {
        11 => format!("{}.{}.{}-{}", &d[..3], &d[3..6], &d[6..9], &d[9..]),
        14 => format!("{}.{}.{}/{}-{}", &d[..2], &d[2..5], &d[5..8], &d[8..12], &d[12..]),
        _ => doc.to_string(),
    }
}

/// Confirmação enviada a quem forneceu CPF/CNPJ após consulta na Receita Federal.
/// `nfse_name` = nome da RF (autoritativo); `fallback_name` = nome do perfil WhatsApp.
pub fn document_confirmed(
    document: &str,
    nfse_name: Option<&str>,
    situation: Option<&str>,
    fallback_name: Option<&str>,
) -> String {
    let doc_fmt = format_document(document);
    let mut lines = vec![format!("✅ *Documento confirmado*\n🪪 {doc_fmt}")];
    if let Some(name) = nfse_name {
        lines.push(format!("👤 Nome (Receita Federal): *{name}*"));
        if let Some(sit) = situation {
            lines.push(format!("📋 Situação cadastral: *{sit}*"));
        }
    } else if let Some(name) = fallback_name {
        lines.push(format!("👤 Nome (WhatsApp): *{name}*"));
        lines.push("_⚠️ Nome da Receita Federal não disponível — configure NFSE\\_INSCRICAO e NFSE\\_SENHA_".into());
    } else {
        lines.push("_⚠️ Nome da Receita Federal não disponível — configure NFSE\\_INSCRICAO e NFSE\\_SENHA_".into());
    }
    lines.join("\n")
}

pub fn welcome() -> &'static str {
    "Olá, eu sou o *HoldFy*. Protejo seu pagamento com segurança."
}

/// Saudação personalizada quando o nome do contacto já é conhecido.
pub fn welcome_known(first_name: &str) -> String {
    format!("Olá, *{first_name}*! Sou o *HoldFy*. Protejo seu pagamento com segurança.")
}

pub fn menu_hint() -> &'static str {
    "Digite *holdfy*, *fazer um holdfy* ou *novo pedido* para começar."
}

pub fn welcome_help() -> &'static str {
    "*Resumo*\n\
     Ex.: *fazer um holdfy de 20 para (41) 99999-9999* — ou envie o *cartão de contacto* do comprador (com ou sem valor na legenda).\n\
     Comprador responde *ACEITO* ou *RECUSO*.\n\
     PIX só depois do aceite; avisamos os dois quando o pagamento for confirmado.\n\n\
     *Comandos:* holdfy, ajuda, cancelar"
}

pub fn start_order_intro() -> &'static str {
    "Vamos criar um *HoldFy*. Informe *valor* e *celular do comprador* (pode ser numa frase só ou em mensagens separadas). O PIX só sai depois do comprador responder *ACEITO*."
}

pub fn ask_counterparty() -> &'static str {
    ask_holdfy_phone()
}

pub fn ask_holdfy_phone() -> &'static str {
    "Para qual número de celular devo enviar o Holdfy?\n\
     Ex.: *(41) 99999-9999* ou *5541999999999* (cartão de contacto também vale)."
}

pub fn invalid_counterparty_phone() -> &'static str {
    invalid_holdfy_phone()
}

pub fn invalid_holdfy_phone() -> &'static str {
    "Esse número parece inválido. Informe um celular com DDD ou envie outro *cartão de contacto*, por exemplo: *(41) 99999-9999*."
}

pub fn buyer_whatsapp_unreachable(masked_buyer: &str) -> String {
    format!(
        "Não consegui enviar a proposta Holdfy para *{masked_buyer}*.\n\
         Confirme se o número tem *WhatsApp* activo e está correcto (com DDD).\n\
         Pode reenviar o *cartão de contacto* do comprador ou o número com DDI *55*."
    )
}

pub fn ask_amount() -> &'static str {
    ask_holdfy_amount()
}

pub fn ask_holdfy_amount() -> &'static str {
    "Qual o *valor* do Holdfy? (ex. *20*, *R$ 20,00*, *20 reais*)"
}

pub fn counterparty_same_as_sender() -> &'static str {
    "O comprador tem de ser *outro* WhatsApp (outro número).\n\
     Não dá para cobrar o *mesmo* número que está a falar com o HoldFy.\n\n\
     Envie *cancelar* e recomece com o telefone de quem vai pagar."
}

/// Proposta ao comprador (B) antes de existir pedido na API.
/// Inclui nome completo e CPF/CNPJ do vendedor (obrigatório).
pub fn buyer_proposal_before_accept(
    seller_phone_masked: &str,
    amount: &str,
    description: &str,
    seller_name: Option<&str>,
    seller_document: &str,
) -> String {
    let seller_doc_fmt = format_document(seller_document);
    let seller_line = match seller_name {
        Some(name) => format!("👤 Vendedor: *{name}* — `{seller_doc_fmt}`"),
        None => format!("👤 Vendedor: *{seller_phone_masked}* — `{seller_doc_fmt}`"),
    };
    format!(
        "*Proposta HoldFy*\n\
         {seller_line}\n\
         💰 Valor: *R$ {amount}*\n\
         📋 Descrição: *{description}*\n\n\
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

pub fn seller_waiting_buyer_must_accept() -> &'static str {
    "O *comprador* precisa responder *ACEITO* no *WhatsApp dele* (não neste chat do vendedor).\n\
     Envie a proposta de novo se ele não recebeu a mensagem."
}

/// Mensagem só com UUID (copiar inteira).
#[must_use]
pub fn order_control_number_only(order_id: &uuid::Uuid) -> String {
    order_id.to_string()
}

/// Vendedor (A): resumo logo após criar pedido, com nome completo e CPF do comprador.
pub fn seller_order_created_notice(
    amount: &str,
    description: &str,
    buyer_phone_masked: &str,
    buyer_name: Option<&str>,
    buyer_document: &str,
) -> String {
    let buyer_doc_fmt = format_document(buyer_document);
    let buyer_line = match buyer_name {
        Some(name) => format!("👤 Comprador: *{name}* — `{buyer_doc_fmt}`"),
        None => format!("👤 Comprador: ~*{buyer_phone_masked}* — `{buyer_doc_fmt}`"),
    };
    format!(
        "*Pedido criado*\n\
         {buyer_line}\n\
         💰 *R$ {amount}*\n\
         📋 *{description}*\n\n\
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

pub fn ask_seller_document() -> &'static str {
    "Antes de enviar a proposta ao comprador, informe *seu* CPF (11 dígitos) ou CNPJ (14 dígitos).\n\
     Consultaremos na Receita Federal para validar os dados.\n\n\
     Ex: `12345678901` ou `12345678000195`"
}

pub fn ask_buyer_document() -> &'static str {
    "Para gerar o PIX, informe *seu* CPF (pessoa física, 11 dígitos) ou *CNPJ* (empresa, 14 dígitos).\n\
     Consultaremos na Receita Federal.\n\n\
     Ex: `12345678901` ou `12345678000195`"
}

pub fn seller_document_pending_before_buyer() -> &'static str {
    "Aguarde: o vendedor ainda precisa informar o CPF/CNPJ dele. Você receberá a proposta em seguida."
}

pub fn invalid_document() -> &'static str {
    "Documento inválido. Envie um *CPF* (11 dígitos) ou *CNPJ* (14 dígitos).\nMáscaras são aceitas (ex.: `123.456.789-09`).\n\nEx: `52998224725`"
}

pub fn invalid_document_retry(attempt: u32, max: u32) -> String {
    format!(
        "Documento não reconhecido (tentativa {attempt}/{max}).\n\
         Envie *CPF* (11 dígitos) ou *CNPJ* (14 dígitos) — máscaras são aceitas.\n\n\
         Ex: `123.456.789-09` ou `52998224725`"
    )
}

pub fn invalid_document_too_many_attempts() -> &'static str {
    "Muitas tentativas inválidas. Fluxo cancelado.\nDigite *holdfy* para recomeçar."
}

pub fn importing_product() -> &'static str {
    "🔍 Buscando produto no link..."
}

pub fn product_imported_with_price(title: &str, price: &str) -> String {
    format!(
        "✅ *{title}*\n💰 R$ *{price}*\n\nPara qual número de celular devo enviar o Holdfy?\nEx.: *(41) 99999-9999*"
    )
}

pub fn product_imported_no_price(title: &str) -> String {
    format!(
        "✅ *{title}*\n\nNão encontrei o preço. Qual o *valor* do Holdfy? (ex. *99,90*)"
    )
}

pub fn product_import_failed() -> &'static str {
    "Não consegui ler esse link. Vamos criar o pedido manualmente.\n\nQual o *valor* e o *celular do comprador*? (pode ser numa frase só)"
}

// ─── Tracking ─────────────────────────────────────────────────────────────────

pub fn tracking_result(code: &str, status: &str, last_event: &str, provider: &str) -> String {
    format!("📦 *{code}*\n*Status:* {status}\n*Último evento:* {last_event}\n_(via {provider})_")
}

pub fn tracking_not_found(code: &str) -> String {
    format!("Código *{code}* não encontrado. Verifique se está correto (ex.: AA123456789BR).")
}

pub fn tracking_all_providers_down() -> &'static str {
    "Serviço de rastreio temporariamente indisponível. Tente novamente em alguns minutos."
}
