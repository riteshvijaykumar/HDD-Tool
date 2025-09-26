use std::env;
use hdd_tool::server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost/hdd_tool".to_string());
    
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    println!("🗄️  Database URL: {}", database_url);
    println!("🚀 Starting HDD Tool Server...");
    
    start_server(database_url, port).await?;
    
    Ok(())
}