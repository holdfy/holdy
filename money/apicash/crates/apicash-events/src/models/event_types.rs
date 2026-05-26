//! Nomes de tópicos e tipos lógicos para roteamento e observabilidade.

/// Sufixo lógico do stream principal de domínio APICash.
pub const MAIN_TOPIC_SUFFIX: &str = "apicash-events";

/// Subscription exclusiva do serviço de custódia.
pub const SUB_CUSTODY: &str = "custody-service";

/// Subscription do pipeline de antifraude.
pub const SUB_ANTIFRAUDE: &str = "antifraude-service";

/// Subscription do processamento de liberação final.
pub const SUB_RELEASE: &str = "release-service";

/// Subscription do worker de importação assíncrona de anúncios.
pub const SUB_IMPORTER: &str = "importer-service";
