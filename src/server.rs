use actix_web::{get, App, HttpResponse, HttpServer, Responder, web};
use anyhow::Result;
use serde::Serialize;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;
use std::sync::Arc;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    status: String,
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Root endpoint", body = String)
    )
)]
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("SHADE server alive")
}

#[utoipa::path(
    get,
    path = "/healthcheck",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
#[get("/healthcheck")]
async fn healthcheck() -> impl Responder {
    let resp = HealthResponse {
        status: "SHADE server running".to_string(),
    };
    HttpResponse::Ok().json(resp)
}

#[derive(OpenApi)]
#[openapi(
    paths(index, healthcheck),
    components(schemas(HealthResponse))
)]
struct ApiDoc;

pub async fn run_server(config_path: &str) -> Result<()> {
    let config = crate::config::Config::load_from_path(config_path)?;
    config.validate()?;

    let storage = create_storage(&config).await?;
    let storage = Arc::new(storage);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    println!("SHADE server running on http://{}", addr);

    // Start socket server if in socket mode
    if matches!(config.storage.mode, crate::config::StorageMode::Socket) {
        let socket_path = config.storage.socket_path.as_ref().unwrap();
        let socket_server = crate::socket::SocketServer::new(socket_path, storage.clone()).await?;
        tokio::spawn(async move {
            if let Err(e) = socket_server.run().await {
                eprintln!("Socket server error: {}", e);
            }
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .service(index)
            .service(healthcheck)
            .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", ApiDoc::openapi()))
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}

async fn create_storage(config: &crate::config::Config) -> Result<Box<dyn crate::storage::StorageBackend>> {
    let database_url = config.storage.database_url.as_ref().unwrap();
    let storage = crate::storage::SqliteStorage::new(database_url).await?;
    Ok(Box::new(storage))
}


