//! # 🦖 ZIK_ZAK Revolution Server
//!
//! The simplest backend server ever created.
//! Pure accounting replaces your entire tech stack.

use anyhow::Result;
use axum::{response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use tokio;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub message: String,
    pub manifesto: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("zik_zak=debug,tower_http=debug")
        .init();

    info!("🦖 Starting ZIK_ZAK Revolution Server");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(revolution_manifesto))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!(
        "🚀 ZIK_ZAK Revolution Server running on http://{}",
        bind_addr
    );
    info!("💀 Backend development is officially DEAD");
    info!("⚡ Pure accounting has replaced your entire tech stack");

    axum::serve(listener, app).await?;

    Ok(())
}

// Revolution manifesto endpoint
async fn revolution_manifesto() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "🦖 Welcome to the ZIK_ZAK Revolution",
        "manifesto": zik_zak::MANIFESTO,
        "version": zik_zak::VERSION,
        "truth": "Backend development is dead. We killed it with divine sparks.",
        "endpoints": {
            "/health": "Check if the revolution is alive",
            "/": "The revolution manifesto"
        }
    }))
}

async fn health_check() -> Json<HealthResponse> {
    let health = HealthResponse {
        status: "🦖 REVOLUTIONARY".to_string(),
        version: zik_zak::VERSION.to_string(),
        message: "The revolution is alive and well!".to_string(),
        manifesto: "Backend development is DEAD! 💀".to_string(),
    };

    Json(health)
}
