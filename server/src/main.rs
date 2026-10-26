use rhystic_cloud_server::{build_router, db::AnalyticalDb};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path =
        std::env::var("RHYSTIC_SERVER_DB").unwrap_or_else(|_| "rhystic_cloud.db".to_string());
    let bind_addr =
        std::env::var("RHYSTIC_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".to_string());

    let db = AnalyticalDb::connect(&db_path).await?;
    let app = build_router(db);

    let addr: SocketAddr = bind_addr.parse()?;
    println!(
        "[Rhystic Cloud Server] Listening on http://{} (DB: {})",
        addr, db_path
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

