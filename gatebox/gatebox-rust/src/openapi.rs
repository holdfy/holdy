// OpenAPI / Swagger (como no Go com swaggo)
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Gatebox API",
        version = "1.0",
        description = "API Gatebox (Rust)",
        contact(name = "API Support", url = "http://www.swagger.io/support", email = "support@swagger.io"),
        license(name = "Apache 2.0", url = "http://www.apache.org/licenses/LICENSE-2.0.html")
    ),
    servers((url = "/api", description = "API base"))
)]
pub struct ApiDoc;
