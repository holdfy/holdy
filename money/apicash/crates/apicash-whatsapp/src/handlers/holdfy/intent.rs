//! Detecção de intenção de criar pedido HoldFy.

/// Intenção reconhecida no canal WhatsApp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldfyIntent {
    CreateHoldfy,
}

fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Verifica se o texto contém a palavra `holdfy` (ou variação próxima).
pub fn contains_holdfy_keyword(text: &str) -> bool {
    let n = normalize(text);
    n.split(|c: char| !c.is_alphanumeric())
        .any(|w| w == "holdfy" || w == "holdfy.")
}

/// Frases legadas ainda aceitas como novo pedido.
pub fn is_legacy_new_order(cmd: &str) -> bool {
    matches!(
        normalize(cmd).as_str(),
        "novo pedido" | "novo pagamento" | "pedido" | "/novo" | "novo_pedido"
    )
}

/// Intenção explícita de criar cobrança HoldFy.
pub fn is_create_holdfy_intent(text: &str) -> bool {
    if is_legacy_new_order(text) {
        return true;
    }
    if !contains_holdfy_keyword(text) {
        return false;
    }
    let n = normalize(text);
    // Apenas "holdfy" ou frases com verbos de ação / cobrança.
    n == "holdfy"
        || n.contains("fazer")
        || n.contains("faça")
        || n.contains("faca")
        || n.contains("quero")
        || n.contains("preciso")
        || n.contains("criar")
        || n.contains("gerar")
        || n.contains("iniciar")
        || n.contains("novo")
        || n.contains("cobrança")
        || n.contains("cobranca")
        || n.contains("pagamento")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synonyms() {
        assert!(is_create_holdfy_intent("fazer um holdfy"));
        assert!(is_create_holdfy_intent("quero fazer um holdfy"));
        assert!(is_create_holdfy_intent("criar holdfy"));
        assert!(is_create_holdfy_intent("gerar cobrança holdfy"));
        assert!(is_create_holdfy_intent("holdfy"));
        assert!(!is_create_holdfy_intent("oi"));
    }
}
