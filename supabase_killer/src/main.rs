//! # 🛡️ ZIK_ZAK REVOLUTIONARY SECURITY MODEL 🛡️
//!
//! COMPLETELY OBLITERATES TRADITIONAL ROW LEVEL SECURITY!
//!
//! ## The Revolution:
//! - Every permission is an account balance
//! - Every user action is a transfer
//! - Security = Balance checks (INSTANT!)
//! - No SQL policies, no complexity
//! - Infinite flexibility, automatic audit trails
//!
//! ## Permission Model:
//! ```
//! user:{user_id}:read:{resource_type}     = 1 (can read)
//! user:{user_id}:write:{resource_type}    = 1 (can write)
//! user:{user_id}:admin                    = 1 (super admin)
//! resource:{id}:owner:{user_id}           = 1 (owns this)
//! tenant:{tenant_id}:member:{user_id}     = 1 (tenant member)
//! ```

use axum::{
    extract::{Path, Query, State, Request},
    http::{StatusCode, HeaderMap},
    response::Json,
    routing::{get, post, delete},
    Router, middleware::{self, Next},
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use uuid::Uuid;

type SharedState = Arc<Mutex<ZikZakSecurityEngine>>;

/// 🦖 The Revolutionary ZIK_ZAK Security Engine
struct ZikZakSecurityEngine {
    // Account balances for permissions and data
    accounts: HashMap<String, i64>,
    // Transaction log for audit trails
    transactions: Vec<SecurityTransaction>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SecurityTransaction {
    id: String,
    from_account: String,
    to_account: String,
    amount: i64,
    operation: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    metadata: HashMap<String, String>,
}

impl ZikZakSecurityEngine {
    fn new() -> Self {
        let mut engine = Self {
            accounts: HashMap::new(),
            transactions: Vec::new(),
        };

        // Initialize system accounts
        engine.accounts.insert("system:genesis".to_string(), 1_000_000);
        engine.accounts.insert("system:void".to_string(), 0);

        engine
    }

    /// 🔥 Core transfer operation - the heart of ZIK_ZAK security
    fn transfer(&mut self, from: &str, to: &str, amount: i64, operation: &str, metadata: HashMap<String, String>) -> Result<String, String> {
        let from_balance = self.accounts.get(from).copied().unwrap_or(0);

        if from_balance < amount {
            return Err(format!("Insufficient balance in {}: {} < {}", from, from_balance, amount));
        }

        // Execute transfer
        *self.accounts.entry(from.to_string()).or_insert(0) -= amount;
        *self.accounts.entry(to.to_string()).or_insert(0) += amount;

        // Log transaction
        let transaction = SecurityTransaction {
            id: Uuid::new_v4().to_string(),
            from_account: from.to_string(),
            to_account: to.to_string(),
            amount,
            operation: operation.to_string(),
            timestamp: chrono::Utc::now(),
            metadata,
        };

        let tx_id = transaction.id.clone();
        self.transactions.push(transaction);

        Ok(tx_id)
    }

    /// ⚡ Lightning-fast permission check (just a balance lookup!)
    fn has_permission(&self, permission_account: &str) -> bool {
        self.accounts.get(permission_account).copied().unwrap_or(0) > 0
    }

    /// 🎯 Extract user ID from authorization header
    fn extract_user_id(headers: &HeaderMap) -> Result<String, String> {
        let auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or("Missing or invalid authorization header")?;

        // In real implementation, decode JWT and extract user ID
        // For demo, we'll parse a simple format: "user_123"
        if auth_header.starts_with("user_") {
            Ok(auth_header.to_string())
        } else {
            Err("Invalid token format".to_string())
        }
    }

    /// 🏗️ Create a new user with automatic permission setup
    fn create_user(&mut self, email: &str, role: &str, tenant_id: Option<&str>) -> Result<String, String> {
        let user_id = format!("user_{}", Uuid::new_v4());

        let mut metadata = HashMap::new();
        metadata.insert("email".to_string(), email.to_string());
        metadata.insert("role".to_string(), role.to_string());
        if let Some(tenant) = tenant_id {
            metadata.insert("tenant_id".to_string(), tenant.to_string());
        }

        // Grant basic permissions based on role
        match role {
            "admin" => {
                // Admins get god mode
                self.transfer("system:genesis", &format!("user:{}:admin", user_id), 1, "grant_admin", metadata.clone())?;
                self.transfer("system:genesis", &format!("user:{}:read:all", user_id), 1, "grant_read_all", metadata.clone())?;
                self.transfer("system:genesis", &format!("user:{}:write:all", user_id), 1, "grant_write_all", metadata.clone())?;
            }
            "customer" => {
                // Customers get basic permissions
                self.transfer("system:genesis", &format!("user:{}:read:products", user_id), 1, "grant_read_products", metadata.clone())?;
                self.transfer("system:genesis", &format!("user:{}:write:orders", user_id), 1, "grant_write_orders", metadata.clone())?;
                self.transfer("system:genesis", &format!("user:{}:read:orders", user_id), 1, "grant_read_orders", metadata.clone())?;
            }
            "manager" => {
                // Managers get elevated permissions
                self.transfer("system:genesis", &format!("user:{}:read:all", user_id), 1, "grant_read_all", metadata.clone())?;
                self.transfer("system:genesis", &format!("user:{}:write:products", user_id), 1, "grant_write_products", metadata.clone())?;
                self.transfer("system:genesis", &format!("user:{}:read:analytics", user_id), 1, "grant_read_analytics", metadata.clone())?;
            }
            _ => {
                return Err("Invalid role".to_string());
            }
        }

        // Add to tenant if specified
        if let Some(tenant) = tenant_id {
            self.transfer("system:genesis", &format!("tenant:{}:member:{}", tenant, user_id), 1, "add_to_tenant", metadata.clone())?;
        }

        // Create user existence
        self.transfer("system:genesis", &format!("user:{}:existence", user_id), 1, "create_user", metadata)?;

        Ok(user_id)
    }

    /// 🎯 Create a resource with ownership
    fn create_resource(&mut self, resource_type: &str, data: Value, owner_id: &str, tenant_id: Option<&str>) -> Result<String, String> {
        let resource_id = Uuid::new_v4().to_string();

        let mut metadata = HashMap::new();
        metadata.insert("resource_type".to_string(), resource_type.to_string());
        metadata.insert("owner_id".to_string(), owner_id.to_string());
        metadata.insert("data".to_string(), data.to_string());
        if let Some(tenant) = tenant_id {
            metadata.insert("tenant_id".to_string(), tenant.to_string());
        }

        // Create resource existence
        self.transfer("system:genesis", &format!("{}:{}:existence", resource_type, resource_id), 1, "create_resource", metadata.clone())?;

        // Set ownership
        self.transfer("system:genesis", &format!("{}:{}:owner:{}", resource_type, resource_id, owner_id), 1, "set_owner", metadata.clone())?;

        // Add to tenant if specified
        if let Some(tenant) = tenant_id {
            self.transfer("system:genesis", &format!("{}:{}:tenant:{}", resource_type, resource_id, tenant), 1, "set_tenant", metadata)?;
        }

        Ok(resource_id)
    }

    /// 🛡️ Check if user can access resource
    fn can_access_resource(&self, user_id: &str, resource_type: &str, resource_id: &str, action: &str) -> bool {
        // Admin override
        if self.has_permission(&format!("user:{}:admin", user_id)) {
            return true;
        }

        // Global permission
        if self.has_permission(&format!("user:{}:{}:all", user_id, action)) {
            return true;
        }

        // Resource type permission
        if self.has_permission(&format!("user:{}:{}:{}", user_id, action, resource_type)) {
            // Check ownership for write operations
            if action == "write" || action == "delete" {
                return self.has_permission(&format!("{}:{}:owner:{}", resource_type, resource_id, user_id));
            }
            return true;
        }

        // Owner can always access their resources
        if self.has_permission(&format!("{}:{}:owner:{}", resource_type, resource_id, user_id)) {
            return true;
        }

        false
    }
}

/// 🛡️ Security middleware - the guardian of ZIK_ZAK
async fn security_middleware(
    State(state): State<SharedState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<axum::response::Response, (StatusCode, Json<Value>)> {
    let path = request.uri().path();

    // Public endpoints that don't need auth
    if path == "/health" || path == "/auth/signup" || path == "/auth/login" || path.starts_with("/public/") {
        return Ok(next.run(request).await);
    }

    // Extract user ID from token
    let user_id = ZikZakSecurityEngine::extract_user_id(&headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(json!({"error": e}))))?;

    // Check if user exists
    let state = state.lock().await;
    if !state.has_permission(&format!("user:{}:existence", user_id)) {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "User not found"}))));
    }

    // For now, allow all authenticated users
    // In real implementation, we'd check specific permissions based on the endpoint
    Ok(next.run(request).await)
}

// 🔐 AUTH ENDPOINTS
async fn auth_signup(
    State(state): State<SharedState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let email = payload["email"].as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "Email required"}))))?;
    let role = payload["role"].as_str().unwrap_or("customer");
    let tenant_id = payload["tenant_id"].as_str();

    let mut state = state.lock().await;

    // Create user with permissions
    let user_id = state.create_user(email, role, tenant_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    Ok(Json(json!({
        "user_id": user_id,
        "access_token": user_id, // Simplified token
        "email": email,
        "role": role,
        "tenant_id": tenant_id,
        "message": "🦖 User created with ZIK_ZAK security!"
    })))
}

async fn auth_login(
    State(state): State<SharedState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let email = payload["email"].as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "Email required"}))))?;

    let state = state.lock().await;

    // Find user by email (simplified lookup)
    let user_id = format!("user_{}", email.replace("@", "_").replace(".", "_"));

    if !state.has_permission(&format!("user:{}:existence", user_id)) {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"}))));
    }

    Ok(Json(json!({
        "user_id": user_id,
        "access_token": user_id,
        "email": email,
        "message": "🦖 Logged in with ZIK_ZAK security!"
    })))
}

// 📊 SECURE RESOURCE ENDPOINTS
async fn create_product(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = ZikZakSecurityEngine::extract_user_id(&headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(json!({"error": e}))))?;

    let mut state = state.lock().await;

    // Check if user can create products
    if !state.can_access_resource(&user_id, "products", "", "write") {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "No permission to create products"}))));
    }

    let tenant_id = payload["tenant_id"].as_str();
    let product_id = state.create_resource("product", payload.clone(), &user_id, tenant_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    Ok(Json(json!({
        "product_id": product_id,
        "owner": user_id,
        "tenant_id": tenant_id,
        "data": payload,
        "message": "🦖 Product created with ZIK_ZAK security!"
    })))
}

async fn get_product(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(product_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = ZikZakSecurityEngine::extract_user_id(&headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(json!({"error": e}))))?;

    let state = state.lock().await;

    // Check if user can read this product
    if !state.can_access_resource(&user_id, "product", &product_id, "read") {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "No permission to read this product"}))));
    }

    // Check if product exists
    if !state.has_permission(&format!("product:{}:existence", product_id)) {
        return Err((StatusCode::NOT_FOUND, Json(json!({"error": "Product not found"}))));
    }

    Ok(Json(json!({
        "product_id": product_id,
        "message": "🦖 Product accessed with ZIK_ZAK security!",
        "permissions": {
            "can_read": state.can_access_resource(&user_id, "product", &product_id, "read"),
            "can_write": state.can_access_resource(&user_id, "product", &product_id, "write"),
            "can_delete": state.can_access_resource(&user_id, "product", &product_id, "delete"),
            "is_owner": state.has_permission(&format!("product:{}:owner:{}", product_id, user_id)),
            "is_admin": state.has_permission(&format!("user:{}:admin", user_id)),
        }
    })))
}

async fn delete_product(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(product_id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = ZikZakSecurityEngine::extract_user_id(&headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(json!({"error": e}))))?;

    let mut state = state.lock().await;

    // Check if user can delete this product
    if !state.can_access_resource(&user_id, "product", &product_id, "delete") {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "No permission to delete this product"}))));
    }

    // Move to void (soft delete)
    let mut metadata = HashMap::new();
    metadata.insert("deleted_by".to_string(), user_id.clone());

    state.transfer(&format!("product:{}:existence", product_id), "system:void", 1, "delete_product", metadata)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    Ok(Json(json!({
        "product_id": product_id,
        "deleted_by": user_id,
        "message": "🦖 Product deleted with ZIK_ZAK security!"
    })))
}

// 🔧 ADMIN ENDPOINTS
async fn grant_permission(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let admin_user_id = ZikZakSecurityEngine::extract_user_id(&headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(json!({"error": e}))))?;

    let mut state = state.lock().await;

    // Only admins can grant permissions
    if !state.has_permission(&format!("user:{}:admin", admin_user_id)) {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Admin access required"}))));
    }

    let target_user_id = payload["user_id"].as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "user_id required"}))))?;
    let permission = payload["permission"].as_str()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, Json(json!({"error": "permission required"}))))?;

    let mut metadata = HashMap::new();
    metadata.insert("granted_by".to_string(), admin_user_id.clone());
    metadata.insert("target_user".to_string(), target_user_id.to_string());

    state.transfer("system:genesis", &format!("user:{}:{}", target_user_id, permission), 1, "grant_permission", metadata)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    Ok(Json(json!({
        "granted_by": admin_user_id,
        "target_user": target_user_id,
        "permission": permission,
        "message": "🦖 Permission granted with ZIK_ZAK security!"
    })))
}

async fn audit_trail(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user_id = ZikZakSecurityEngine::extract_user_id(&headers)
        .map_err(|e| (StatusCode::UNAUTHORIZED, Json(json!({"error": e}))))?;

    let state = state.lock().await;

    // Only admins can view audit trails
    if !state.has_permission(&format!("user:{}:admin", user_id)) {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "Admin access required"}))));
    }

    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(50);

    let recent_transactions: Vec<_> = state.transactions
        .iter()
        .rev()
        .take(limit)
        .collect();

    Ok(Json(json!({
        "transactions": recent_transactions,
        "total_count": state.transactions.len(),
        "message": "🦖 Audit trail retrieved with ZIK_ZAK security!"
    })))
}

async fn security_stats(
    State(state): State<SharedState>,
) -> Json<Value> {
    let state = state.lock().await;

    let total_accounts = state.accounts.len();
    let total_transactions = state.transactions.len();
    let total_permissions = state.accounts.iter()
        .filter(|(k, v)| k.contains(":") && **v > 0)
        .count();

    Json(json!({
        "status": "🦖 ZIK_ZAK SECURITY OPERATIONAL",
        "total_accounts": total_accounts,
        "total_transactions": total_transactions,
        "total_permissions": total_permissions,
        "security_model": "ACCOUNTING-BASED",
        "performance": "INSTANT PERMISSION CHECKS",
        "audit_trail": "AUTOMATIC",
        "flexibility": "INFINITE",
        "message": "TRADITIONAL ROW LEVEL SECURITY IS DEAD!"
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,supabase_killer=debug")
        .init();

    info!("🚀 Starting ZIK_ZAK REVOLUTIONARY SECURITY server...");

    let state = Arc::new(Mutex::new(ZikZakSecurityEngine::new()));

    let app = Router::new()
        // 🔐 Auth endpoints (no middleware)
        .route("/auth/signup", post(auth_signup))
        .route("/auth/login", post(auth_login))

        // 📊 Secured resource endpoints
        .route("/products", post(create_product))
        .route("/products/:id", get(get_product))
        .route("/products/:id", delete(delete_product))

        // 🔧 Admin endpoints
        .route("/admin/grant-permission", post(grant_permission))
        .route("/admin/audit-trail", get(audit_trail))

        // 🚀 Public endpoints
        .route("/security/stats", get(security_stats))
        .route("/health", get(|| async { Json(json!({"status": "🦖 ZIK_ZAK SECURITY ALIVE"})) }))

        .layer(middleware::from_fn_with_state(state.clone(), security_middleware))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    // Bind to security port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:54321").await?;

    info!("🛡️  ZIK_ZAK REVOLUTIONARY SECURITY listening on port 54321");
    info!("🔥 TRADITIONAL ROW LEVEL SECURITY IS DEAD!");
    info!("⚡ PERMISSION CHECKS ARE NOW INSTANT!");
    info!("🎯 INFINITE FLEXIBILITY ACHIEVED!");

    axum::serve(listener, app).await?;

    Ok(())
}
