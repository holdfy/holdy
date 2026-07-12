//! Catálogo de mensagens por idioma.

use super::Locale;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgKey {
    // Idioma / navegação
    Language,
    LangPt,
    LangEn,
    LangEs,
    Brand,
    NavDashboard,
    NavOrders,
    NavDisputes,
    NavSellers,
    NavReports,
    NavStellar,
    NavbarSubtitle,
    Logout,
    PageTitle,
    PageNotFound,

    // Comum
    Loading,
    LoadingOrders,
    LoadingDisputes,
    LoadingYield,
    ErrorApi,
    Detail,
    Dash,

    // Dashboard
    DashboardTitle,
    DashboardSubtitle,
    DashboardYieldSection,
    CardTotalVolume,
    CardAccumulatedYield,
    CardOpenDisputes,
    CardAvgScore,
    CardLockedCustodies,

    // Pedidos
    OrdersTitle,
    OrdersSubtitle,
    ColOrder,
    ColStatus,
    ColAmount,
    ColScore,
    ColDecision,
    ColPlatform,

    // Disputas
    DisputesTitle,
    DisputesSubtitle,
    ColId,
    ColReason,
    ResolveManual,

    // Vendedores
    SellersTitle,
    SellersSubtitle,
    SellersPlaceholder,
    SellersLoad,
    SellersPromptUuid,
    CardOrders,
    CardVolume,

    // Relatórios
    ReportsTitle,
    ReportsSubtitle,
    ReportsYield,
    ReportsScores,
    ReportsPrepareCsv,

    // Yield
    YieldTotalReported,
    YieldCustodies,

    // Stellar
    StellarTitle,
    StellarSubtitle,
    StellarBack,
    StellarNetwork,
    StellarMode,
    StellarContract,
    StellarLockTx,
    StellarReleaseTx,
    StellarBrlxTx,
    StellarOpenExplorer,
    StellarNegotiation,
    StellarBlockchain,
    StellarBuyer,
    StellarDocument,
    StellarSeller,
    StellarAmount,
    StellarOrderStatus,
    StellarCustodyStatus,
    StellarCreatedAt,
    StellarNoData,
    LoadingStellar,
    ColNetwork,
    ColMode,
    ColLockTx,

    // Auth
    DefaultAdminUser,
}

macro_rules! msg {
    ($pt:expr, $en:expr, $es:expr) => {
        [ $pt, $en, $es ]
    };
}

type Triple = [&'static str; 3];

const TABLE: &[(MsgKey, Triple)] = &[
    (MsgKey::Language, msg!("Idioma", "Language", "Idioma")),
    (MsgKey::LangPt, msg!("Português (BR)", "Portuguese (BR)", "Portugués (BR)")),
    (MsgKey::LangEn, msg!("Inglês", "English", "Inglés")),
    (MsgKey::LangEs, msg!("Espanhol", "Spanish", "Español")),
    (MsgKey::Brand, msg!("HoldFy", "HoldFy", "HoldFy")),
    (MsgKey::NavDashboard, msg!("Dashboard", "Dashboard", "Panel")),
    (MsgKey::NavOrders, msg!("Pedidos", "Orders", "Pedidos")),
    (MsgKey::NavDisputes, msg!("Disputas", "Disputes", "Disputas")),
    (MsgKey::NavSellers, msg!("Vendedores", "Sellers", "Vendedores")),
    (MsgKey::NavReports, msg!("Relatórios", "Reports", "Informes")),
    (MsgKey::NavStellar, msg!("Stellar", "Stellar", "Stellar")),
    (
        MsgKey::NavbarSubtitle,
        msg!(
            "Painel interno HoldFy",
            "HoldFy internal dashboard",
            "Panel interno HoldFy"
        ),
    ),
    (MsgKey::Logout, msg!("Sair", "Log out", "Salir")),
    (MsgKey::PageTitle, msg!("HoldFy Admin", "HoldFy Admin", "HoldFy Admin")),
    (
        MsgKey::PageNotFound,
        msg!("Página não encontrada.", "Page not found.", "Página no encontrada."),
    ),
    (MsgKey::Loading, msg!("A carregar…", "Loading…", "Cargando…")),
    (
        MsgKey::LoadingOrders,
        msg!("A carregar pedidos…", "Loading orders…", "Cargando pedidos…"),
    ),
    (
        MsgKey::LoadingDisputes,
        msg!("A carregar disputas…", "Loading disputes…", "Cargando disputas…"),
    ),
    (
        MsgKey::LoadingYield,
        msg!("A carregar yield…", "Loading yield…", "Cargando rendimiento…"),
    ),
    (
        MsgKey::ErrorApi,
        msg!("Erro API: ", "API error: ", "Error de API: "),
    ),
    (MsgKey::Detail, msg!("detalhe", "detail", "detalle")),
    (MsgKey::Dash, msg!("—", "—", "—")),
    (
        MsgKey::DashboardTitle,
        msg!("Dashboard", "Dashboard", "Panel"),
    ),
    (
        MsgKey::DashboardSubtitle,
        msg!(
            "Resumo operacional e indicadores de risco.",
            "Operational summary and risk indicators.",
            "Resumen operativo e indicadores de riesgo."
        ),
    ),
    (
        MsgKey::DashboardYieldSection,
        msg!("Yield (visualização)", "Yield (visualization)", "Rendimiento (visualización)"),
    ),
    (
        MsgKey::CardTotalVolume,
        msg!("Volume total", "Total volume", "Volumen total"),
    ),
    (
        MsgKey::CardAccumulatedYield,
        msg!("Yield acumulado", "Accumulated yield", "Rendimiento acumulado"),
    ),
    (
        MsgKey::CardOpenDisputes,
        msg!("Disputas abertas", "Open disputes", "Disputas abiertas"),
    ),
    (
        MsgKey::CardAvgScore,
        msg!("Score médio", "Average score", "Puntuación media"),
    ),
    (
        MsgKey::CardLockedCustodies,
        msg!("Custódias travadas", "Locked custodies", "Custodias bloqueadas"),
    ),
    (MsgKey::OrdersTitle, msg!("Pedidos", "Orders", "Pedidos")),
    (
        MsgKey::OrdersSubtitle,
        msg!(
            "Lista com estado, valor e score de risco (origem: admin API).",
            "List with status, amount and risk score (source: admin API).",
            "Lista con estado, valor y puntuación de riesgo (origen: admin API)."
        ),
    ),
    (MsgKey::ColOrder, msg!("Pedido", "Order", "Pedido")),
    (MsgKey::ColStatus, msg!("Estado", "Status", "Estado")),
    (MsgKey::ColAmount, msg!("Valor", "Amount", "Valor")),
    (MsgKey::ColScore, msg!("Score", "Score", "Puntuación")),
    (MsgKey::ColDecision, msg!("Decisão", "Decision", "Decisión")),
    (MsgKey::ColPlatform, msg!("Plataforma", "Platform", "Plataforma")),
    (MsgKey::DisputesTitle, msg!("Disputas", "Disputes", "Disputas")),
    (
        MsgKey::DisputesSubtitle,
        msg!(
            "Gestão de disputas; resolução manual preparada.",
            "Dispute management; manual resolution ready.",
            "Gestión de disputas; resolución manual preparada."
        ),
    ),
    (MsgKey::ColId, msg!("ID", "ID", "ID")),
    (MsgKey::ColReason, msg!("Motivo", "Reason", "Motivo")),
    (
        MsgKey::ResolveManual,
        msg!("Resolver (manual)", "Resolve (manual)", "Resolver (manual)"),
    ),
    (MsgKey::SellersTitle, msg!("Vendedores", "Sellers", "Vendedores")),
    (
        MsgKey::SellersSubtitle,
        msg!(
            "UUID do vendedor → GET /admin/sellers/:id/dashboard",
            "Seller UUID → GET /admin/sellers/:id/dashboard",
            "UUID del vendedor → GET /admin/sellers/:id/dashboard"
        ),
    ),
    (
        MsgKey::SellersPlaceholder,
        msg!("UUID do vendedor", "Seller UUID", "UUID del vendedor"),
    ),
    (MsgKey::SellersLoad, msg!("Carregar", "Load", "Cargar")),
    (
        MsgKey::SellersPromptUuid,
        msg!(
            "Indique um UUID e clique em Carregar.",
            "Enter a UUID and click Load.",
            "Indique un UUID y haga clic en Cargar."
        ),
    ),
    (MsgKey::CardOrders, msg!("Pedidos", "Orders", "Pedidos")),
    (MsgKey::CardVolume, msg!("Volume", "Volume", "Volumen")),
    (MsgKey::ReportsTitle, msg!("Relatórios", "Reports", "Informes")),
    (
        MsgKey::ReportsSubtitle,
        msg!(
            "Exportação CSV/PDF pode ligar-se a estas server functions.",
            "CSV/PDF export can connect to these server functions.",
            "La exportación CSV/PDF puede conectarse a estas server functions."
        ),
    ),
    (MsgKey::ReportsYield, msg!("Yield", "Yield", "Rendimiento")),
    (MsgKey::ReportsScores, msg!("Scores", "Scores", "Puntuaciones")),
    (
        MsgKey::ReportsPrepareCsv,
        msg!(
            "Preparar CSV (copiar no próximo passo)",
            "Prepare CSV (copy in next step)",
            "Preparar CSV (copiar en el siguiente paso)"
        ),
    ),
    (
        MsgKey::YieldTotalReported,
        msg!("Total reportado: ", "Total reported: ", "Total reportado: "),
    ),
    (
        MsgKey::YieldCustodies,
        msg!(" · custódias: ", " · custodies: ", " · custodias: "),
    ),
    (
        MsgKey::DefaultAdminUser,
        msg!("Administrador", "Administrator", "Administrador"),
    ),

    // Stellar
    (MsgKey::StellarTitle,   msg!("Transações Stellar", "Stellar Transactions", "Transacciones Stellar")),
    (MsgKey::StellarSubtitle, msg!(
        "On-chain: hashes Soroban, contratos escrow e trilha BRLx por pedido.",
        "On-chain: Soroban hashes, escrow contracts and BRLx trail per order.",
        "On-chain: hashes Soroban, contratos escrow y trazabilidad BRLx."
    )),
    (MsgKey::StellarBack,    msg!("← Voltar à lista", "← Back to list", "← Volver a la lista")),
    (MsgKey::StellarNetwork, msg!("Rede", "Network", "Red")),
    (MsgKey::StellarMode,    msg!("Modo Soroban", "Soroban Mode", "Modo Soroban")),
    (MsgKey::StellarContract,msg!("Contrato Escrow", "Escrow Contract", "Contrato Escrow")),
    (MsgKey::StellarLockTx,  msg!("TX Lock (PIX→Escrow)", "TX Lock (PIX→Escrow)", "TX Lock (PIX→Escrow)")),
    (MsgKey::StellarReleaseTx,msg!("TX Release (liberação)", "TX Release", "TX Release (liberación)")),
    (MsgKey::StellarBrlxTx, msg!("TX BRLx Transfer", "TX BRLx Transfer", "TX BRLx Transfer")),
    (MsgKey::StellarOpenExplorer, msg!("↗ Ver no Stellar Expert", "↗ Open in Stellar Expert", "↗ Ver en Stellar Expert")),
    (MsgKey::StellarNegotiation, msg!("Negociação", "Order Details", "Negociación")),
    (MsgKey::StellarBlockchain,  msg!("Blockchain Stellar", "Stellar Blockchain", "Blockchain Stellar")),
    (MsgKey::StellarBuyer,   msg!("Comprador", "Buyer", "Comprador")),
    (MsgKey::StellarDocument,msg!("Documento", "Document", "Documento")),
    (MsgKey::StellarSeller,  msg!("Vendedor", "Seller", "Vendedor")),
    (MsgKey::StellarAmount,  msg!("Valor", "Amount", "Valor")),
    (MsgKey::StellarOrderStatus, msg!("Status do pedido", "Order status", "Estado del pedido")),
    (MsgKey::StellarCustodyStatus, msg!("Status da custódia", "Custody status", "Estado de custodia")),
    (MsgKey::StellarCreatedAt, msg!("Criado em", "Created at", "Creado el")),
    (MsgKey::StellarNoData,  msg!("Nenhuma transação Stellar encontrada (Postgres inativo ou sem pedidos com on-ramp).", "No Stellar transactions found (Postgres inactive or no on-ramp orders).", "No se encontraron transacciones Stellar.")),
    (MsgKey::LoadingStellar, msg!("A carregar transações Stellar…", "Loading Stellar transactions…", "Cargando transacciones Stellar…")),
    (MsgKey::ColNetwork,     msg!("Rede", "Network", "Red")),
    (MsgKey::ColMode,        msg!("Modo", "Mode", "Modo")),
    (MsgKey::ColLockTx,      msg!("Lock TX", "Lock TX", "Lock TX")),
];

fn locale_index(locale: Locale) -> usize {
    match locale {
        Locale::PtBr => 0,
        Locale::En => 1,
        Locale::Es => 2,
    }
}

pub fn lookup(locale: Locale, key: MsgKey) -> &'static str {
    let idx = locale_index(locale);
    TABLE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, triple)| triple[idx])
        .unwrap_or("???")
}
