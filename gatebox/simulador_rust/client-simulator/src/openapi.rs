// OpenAPI / Swagger para Client Simulator
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Client Simulator API",
        version = "1.0",
        description = "API para simular transações PIX e teste de carga"
    )
)]
pub struct ApiDoc;
