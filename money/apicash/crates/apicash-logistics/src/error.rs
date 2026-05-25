use thiserror::Error;

#[derive(Debug, Error)]
pub enum LogisticsError {
    #[error("Token da Melhor Envio ausente (MELHOR_ENVIO_TOKEN)")]
    MissingToken,
    #[error("Erro HTTP da Melhor Envio: {0}")]
    ApiError(String),
    #[error("Falha na requisição: {0}")]
    RequestFailed(String),
    #[error("Código de rastreio não encontrado: {0}")]
    TrackingNotFound(String),
    #[error("Endereço inválido: {0}")]
    InvalidAddress(String),
    #[error("Todos os provedores de rastreio falharam para o código: {0}")]
    AllProvidersUnavailable(String),
}
