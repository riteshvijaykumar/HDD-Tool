use warp::Filter;
use std::sync::Arc;
use uuid::Uuid;
use crate::server::{DatabaseManager, models::*};

pub async fn start_server(database_url: String, port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = Arc::new(DatabaseManager::new(&database_url).await?);
    
    // CORS configuration
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);
    
    // Routes
    let register = warp::path("api")
        .and(warp::path("register"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(register_user);
    
    let login = warp::path("api")
        .and(warp::path("login"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(login_user);
    
    let submit_cert = warp::path("api")
        .and(warp::path("certificates"))
        .and(warp::post())
        .and(warp::header::<String>("authorization"))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(submit_certificate);
    
    let get_certs = warp::path("api")
        .and(warp::path("certificates"))
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<PaginationQuery>())
        .and(with_db(db.clone()))
        .and_then(get_certificates);
    
    let get_logs = warp::path("api")
        .and(warp::path("logs"))
        .and(warp::get())
        .and(warp::header::<String>("authorization"))
        .and(warp::query::<PaginationQuery>())
        .and(with_db(db.clone()))
        .and_then(get_sanitization_logs);
    
    // Static files for web dashboard
    let static_files = warp::path("dashboard")
        .and(warp::fs::dir("web/"));
    
    let routes = register
        .or(login)
        .or(submit_cert)
        .or(get_certs)
        .or(get_logs)
        .or(static_files)
        .with(cors);
    
    println!("🚀 HDD Tool Server starting on port {}", port);
    println!("📊 Dashboard available at: http://localhost:{}/dashboard", port);
    println!("🔗 API endpoints:");
    println!("   POST /api/register - Create user account");
    println!("   POST /api/login - User login");
    println!("   POST /api/certificates - Submit certificate");
    println!("   GET  /api/certificates - Get user certificates");
    println!("   GET  /api/logs - Get sanitization logs");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
        
    Ok(())
}

fn with_db(db: Arc<DatabaseManager>) -> impl Filter<Extract = (Arc<DatabaseManager>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

#[derive(serde::Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 { 50 }

// Extract user ID from Bearer token (simplified - in production use JWT)
fn extract_user_id(auth_header: &str) -> Result<Uuid, String> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Uuid::parse_str(token).map_err(|_| "Invalid token format".to_string())
    } else {
        Err("Invalid authorization header".to_string())
    }
}

async fn register_user(
    req: CreateUserRequest,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db.create_user(req).await {
        Ok(user) => {
            let response = ApiResponse::success(LoginResponse {
                token: user.id.to_string(), // Simplified - use JWT in production
                user_id: user.id,
                username: user.username,
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(format!("Registration failed: {}", e));
            Ok(warp::reply::json(&response))
        }
    }
}

async fn login_user(
    req: LoginRequest,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match db.authenticate_user(req).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(LoginResponse {
                token: user.id.to_string(), // Simplified - use JWT in production
                user_id: user.id,
                username: user.username,
            });
            Ok(warp::reply::json(&response))
        }
        Ok(None) => {
            let response: ApiResponse<()> = ApiResponse::error("Invalid credentials".to_string());
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(format!("Login failed: {}", e));
            Ok(warp::reply::json(&response))
        }
    }
}

async fn submit_certificate(
    auth_header: String,
    req: SubmitCertificateRequest,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            match db.store_certificate(user_id, req).await {
                Ok(certificate) => {
                    let response = ApiResponse::success(certificate);
                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::error(format!("Failed to store certificate: {}", e));
                    Ok(warp::reply::json(&response))
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(e);
            Ok(warp::reply::json(&response))
        }
    }
}

async fn get_certificates(
    auth_header: String,
    query: PaginationQuery,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            match db.get_user_certificates(user_id, query.limit, query.offset).await {
                Ok(certificates) => {
                    let response = ApiResponse::success(certificates);
                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::error(format!("Failed to get certificates: {}", e));
                    Ok(warp::reply::json(&response))
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(e);
            Ok(warp::reply::json(&response))
        }
    }
}

async fn get_sanitization_logs(
    auth_header: String,
    query: PaginationQuery,
    db: Arc<DatabaseManager>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match extract_user_id(&auth_header) {
        Ok(user_id) => {
            match db.get_user_logs(user_id, query.limit, query.offset).await {
                Ok(logs) => {
                    let response = ApiResponse::success(logs);
                    Ok(warp::reply::json(&response))
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::error(format!("Failed to get logs: {}", e));
                    Ok(warp::reply::json(&response))
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::error(e);
            Ok(warp::reply::json(&response))
        }
    }
}