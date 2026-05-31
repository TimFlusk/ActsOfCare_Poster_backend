mod db;
mod storage;
mod routes;
mod models;

use std::sync::Arc;
use axum::{Router, routing::post};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use db::Database;
use storage::MinioStorage;

pub struct AppState {
    pub db: Database,
    pub storage: MinioStorage,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "tng_backend=debug,info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Database::new("tng.db")?;
    let storage = MinioStorage::new()?;

    storage.ensure_bucket().await?;

    let state = Arc::new(AppState { db, storage });

    let app = Router::new()
        .route("/user", post(routes::upsert_user))
        .route("/image", post(routes::upload_image))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
