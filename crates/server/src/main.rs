use anyhow::Result;
use echo_service::{get_router, AppConfig};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let app = get_router(AppConfig::default()).await;
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
