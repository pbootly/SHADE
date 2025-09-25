use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

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

#[utoipa::path(
    get,
    path = "/ip",
    responses(
        (status = 200, description = "Returns the client's IP address", body = String)
    )
)]
#[get("/client-ip")]
async fn return_client_ip(req: actix_web::HttpRequest) -> impl Responder {
    if let Some(peer_addr) = req.peer_addr() {
        HttpResponse::Ok().body(peer_addr.ip().to_string())
    } else {
        HttpResponse::InternalServerError().body("Unable to determine client IP")
    }
}

#[utoipa::path(
    post,
    path = "/register",
    request_body(content = crate::models::RegisterRequest, description = "Request body containing public_key"),
    responses(
        (status = 200, description = "Registers the client's IP address", body = String),
        (status = 400, description = "Invalid public_key or missing IP address"),
        (status = 500, description = "Unable to register the IP address")
    )
)]
#[post("/register")]
async fn register_client_ip(
    req: actix_web::HttpRequest,
    body: web::Json<crate::models::RegisterRequest>,
    storage: web::Data<Arc<dyn crate::storage::StorageBackend>>,
) -> impl Responder {
    let public_key = &body.public_key;

    // Validate public_key exists in the database
    if !storage.validate_public_key(public_key).await.unwrap_or(false) {
        return HttpResponse::BadRequest().body("Invalid public_key");
    }

    // Register client IP
    if let Some(peer_addr) = req.peer_addr() {
        let ip_address = peer_addr.ip().to_string();
        if let Err(e) = storage.store_client_ip(ip_address.clone()).await {
            eprintln!("Failed to store IP address: {}", e);
            return HttpResponse::InternalServerError().body("Unable to register the IP address");
        }
        HttpResponse::Ok().body("IP address registered successfully")
    } else {
        HttpResponse::InternalServerError().body("Unable to determine client IP")
    }
}

#[derive(OpenApi)]
#[openapi(paths(index, healthcheck), components(schemas(HealthResponse)))]
struct ApiDoc;

pub async fn run_server(config_path: &str) -> Result<()> {
    let config = crate::config::Config::load(config_path)?;
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
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(&addr)?
    .run()
    .await?;

    Ok(())
}

async fn create_storage(
    config: &crate::config::Config,
) -> Result<Box<dyn crate::storage::StorageBackend>> {
    let database_url = config.storage.database_url.as_ref().unwrap();
    let storage = crate::storage::SqliteStorage::new(database_url).await?;
    Ok(Box::new(storage))
}
