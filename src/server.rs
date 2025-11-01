use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
#[tracing::instrument(name = "healthcheck")]
async fn healthcheck() -> impl Responder {
    let resp = crate::models::HealthResponse {
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
#[tracing::instrument(name = "ip", skip(req))]
#[get("/ip")]
async fn return_client_ip(req: actix_web::HttpRequest) -> impl Responder {
    match return_ip(&req) {
        Some((source, ip)) => {
            info!("client IP determined via {}: {}", source, ip);
            HttpResponse::Ok().body(ip)
        }
        None => {
            error!("unable to determine client IP");
            HttpResponse::InternalServerError().body("Unable to determine client IP")
        }
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
#[tracing::instrument(name = "register", skip(req))]
#[post("/register")]
async fn register_client_ip(
    req: actix_web::HttpRequest,
    body: web::Json<crate::models::RegisterRequest>,
    storage: web::Data<Arc<dyn crate::storage::StorageBackend>>,
) -> impl Responder {
    let public_key = &body.public_key;

    // Validate public_key exists in the database
    info!("validating public key");
    info!(storage = ?storage);
    if !storage
        .validate_public_key(public_key)
        .await
        .unwrap_or(false)
    {
        error!("public key attempted but not found");
        return HttpResponse::BadRequest().body("Invalid public_key");
    }

    // Register client IP
    match return_ip(&req) {
        Some((_source, ip)) => {
            info!("registering client: {}", ip);
            if let Err(e) = storage.store_client_ip(ip.clone()).await {
                error!("Failed to store IP: {}", e);
                return HttpResponse::InternalServerError().body("Failed to store IP");
            }
            let resp = crate::models::RegisterResponse {
                message: format!("IP {} registered successfully", ip),
            };
            HttpResponse::Ok().json(resp)
        }
        None => {
            error!("IP registration failed. Please try again");
            HttpResponse::InternalServerError().body("something went wrong")
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(index, healthcheck, return_client_ip, register_client_ip),
    components(schemas(crate::models::HealthResponse, crate::models::RegisterRequest))
)]
struct ApiDoc;

pub async fn run_server(config_path: &str) -> Result<()> {
    let config = crate::config::Config::load(config_path)?;
    config.validate()?;

    let storage = create_storage(&config).await?;
    let storage = Arc::clone(&storage);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    info!("SHADE server running on http://{}", addr);

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
            .service(return_client_ip)
            .service(register_client_ip)
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

pub async fn create_storage(
    config: &crate::config::Config,
) -> Result<Arc<dyn crate::storage::StorageBackend>> {
    let database_url = config.storage.database_url.as_ref().unwrap();

    // Wrap the concrete SqliteStorage in Arc and coerce to trait object
    let storage: Arc<dyn crate::storage::StorageBackend> =
        Arc::new(crate::storage::SqliteStorage::new(database_url).await?);

    Ok(storage)
}

fn return_ip(req: &actix_web::HttpRequest) -> Option<(&str, String)> {
    // Attempt to extract the client IP from headers or peer address
    (|| {
        // Check common proxy headers first
        if let Some(forwarded) = req.headers().get("x-forwarded-for")
            && let Ok(forwarded_str) = forwarded.to_str()
            && let Some(ip) = forwarded_str.split(',').next()
        {
            return Some(("x-forwarded-for", ip.trim().to_string()));
        }

        if let Some(forwarded) = req.headers().get("forwarded")
            && let Ok(forwarded_str) = forwarded.to_str()
            && let Some(ip) = forwarded_str.split('=').nth(1)
        {
            return Some(("forwarded", ip.trim().to_string()));
        }

        // Fallback to peer address
        req.peer_addr()
            .map(|addr| ("peer_addr", addr.ip().to_string()))
    })()
}
