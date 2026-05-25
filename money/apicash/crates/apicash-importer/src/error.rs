use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImporterError {
    #[error("URL inválida ou não suportada: {0}")]
    InvalidUrl(String),
    #[error("Falha ao buscar a página: {0}")]
    FetchFailed(String),
    #[error("Nenhum extrator conseguiu extrair dados do produto")]
    NoDataExtracted,
    #[error("Erro na API do MercadoLivre: {0}")]
    MercadoLivreApi(String),
    #[error("Erro na API do LLM: {0}")]
    LlmApi(String),
    #[error("Erro de serialização: {0}")]
    Serialization(String),
}
