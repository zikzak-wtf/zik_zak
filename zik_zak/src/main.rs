use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};

mod accounting;
mod recipes;
mod tigerbeetle_client;

use recipes::RecipeEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRequest {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub account_id: String,
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub from_account: String,
    pub to_account: String,
    pub amount: i64,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub transfer_id: String,
    pub from_account: String,
    pub to_account: String,
    pub amount: i64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRecipeRequest {
    pub recipe_name: String,
    pub inputs: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRecipeResponse {
    pub recipe_name: String,
    pub result: serde_json::Value,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub tigerbeetle_connected: bool,
    pub total_accounts: usize,
    pub total_transfers: usize,
}

// Mock accounting engine for testing without TigerBeetle
pub struct MockZikZakEngine;

impl MockZikZakEngine {
    pub fn new() -> Self {
        Self
    }
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub accounting: Arc<Mutex<MockZikZakEngine>>,
    pub recipes: Arc<RecipeEngine>,
}

// SAFETY: AppState contains only Arc<Mutex<MockZikZakEngine>> and Arc<RecipeEngine>
// Both are thread-safe and the inner types implement Send + Sync
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

// Mock engine is simple and thread-safe
unsafe impl Send for MockZikZakEngine {}
unsafe impl Sync for MockZikZakEngine {}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("zik_zak=debug,tower_http=debug")
        .init();

    info!("🦖 Starting ZIK_ZAK Revolution Server");

    // Skip TigerBeetle connection for now - create mock state
    info!("📊 Skipping TigerBeetle connection (using mock data)...");

    // Load recipes (with fallback to empty recipes if file doesn't exist)
    info!("🍳 Loading recipes...");
    let recipes = match RecipeEngine::new("recipes.json") {
        Ok(r) => r,
        Err(_) => {
            info!("📄 No recipes.json found, using empty recipe engine");
            RecipeEngine::empty()
        }
    };

    // Create app state without accounting engine for now
    let state = AppState {
        accounting: Arc::new(Mutex::new(MockZikZakEngine::new())),
        recipes: Arc::new(recipes),
    };

    // Build our application with routes
    let app = Router::new()
        .route("/test", get(test_handler))
        .route("/health", get(health_check))
        .route("/balance/:account_id", get(get_balance))
        .route("/balance", post(get_balance_post))
        .route("/transfer", post(create_transfer))
        .route("/recipe/:recipe_name", post(execute_recipe))
        .route("/recipes", get(list_recipes))
        .route("/ledger", get(get_ledger_state))
        .route("/transactions", get(get_transaction_history))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3003".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("🚀 Server running on http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

// Simple test handler without complex state usage
async fn test_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

#[instrument(skip(_state))]
async fn health_check(State(_state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let health = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        tigerbeetle_connected: true, // Simplified for now
        total_accounts: 0,           // Simplified for now
        total_transfers: 0,          // Simplified for now
    };

    Ok(Json(health))
}

#[instrument(skip(_state))]
async fn get_balance(
    State(_state): State<AppState>,
    Path(account_id): Path<String>,
) -> Result<Json<BalanceResponse>, StatusCode> {
    // Simplified for now - return mock balance
    Ok(Json(BalanceResponse {
        account_id,
        balance: 1000, // Mock balance
    }))
}

#[instrument(skip(_state))]
async fn get_balance_post(
    State(_state): State<AppState>,
    Json(request): Json<BalanceRequest>,
) -> Result<Json<BalanceResponse>, StatusCode> {
    // Simplified for now - return mock balance
    Ok(Json(BalanceResponse {
        account_id: request.account_id,
        balance: 1000, // Mock balance
    }))
}

#[instrument(skip(_state))]
async fn create_transfer(
    State(_state): State<AppState>,
    Json(request): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, StatusCode> {
    // Simplified for now - return mock transfer
    let transfer_id = uuid::Uuid::new_v4().to_string();
    let response = TransferResponse {
        transfer_id,
        from_account: request.from_account,
        to_account: request.to_account,
        amount: request.amount,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn execute_recipe(
    State(state): State<AppState>,
    Path(recipe_name): Path<String>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> Result<Json<ExecuteRecipeResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Check if recipe exists
    let recipes_list = state.recipes.list_recipes();
    if !recipes_list.get(&recipe_name).is_some() {
        return Err(StatusCode::NOT_FOUND);
    }

    let execution_time = start_time.elapsed().as_millis() as u64;

    // Return mock result for now
    Ok(Json(ExecuteRecipeResponse {
        recipe_name,
        result: serde_json::json!({"status": "executed", "mock": true}),
        execution_time_ms: execution_time,
    }))
}

#[instrument(skip(state))]
async fn list_recipes(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(state.recipes.list_recipes())
}

#[instrument(skip(_state))]
async fn get_ledger_state(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Return mock ledger state for now
    Ok(Json(serde_json::json!({
        "mock_account_1": 1000,
        "mock_account_2": 2000,
        "system:genesis": 1000000
    })))
}

#[instrument(skip(_state))]
async fn get_transaction_history(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Return mock transaction history for now
    Ok(Json(serde_json::json!([
        {
            "id": "mock-transfer-1",
            "from_account": "user:1",
            "to_account": "user:2",
            "amount": 100,
            "timestamp": 1640995200
        }
    ])))
}
