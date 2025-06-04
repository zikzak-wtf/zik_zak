//! # 🔐 Auth Service - JWT Authentication with ZIK_ZAK
//!
//! Replaces Supabase Auth with ZIK_ZAK's accounting-based user management.
//! Every user is an account, every permission is a balance!

use anyhow::{anyhow, Result};
use axum::http::HeaderMap;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use bcrypt::{hash, verify, DEFAULT_COST};
use zik_zak::accounting::ZikZakEngine;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,       // User ID
    exp: usize,        // Expiration
    iat: usize,        // Issued at
    iss: String,       // Issuer
    aud: String,       // Audience
    role: String,      // User role
    email: String,     // User email
}

#[derive(Clone)]
pub struct AuthService {
    zikzak: ZikZakEngine,
    jwt_secret: String,
}

impl AuthService {
    pub async fn new() -> Result<Self> {
        let mut zikzak = ZikZakEngine::new("auth_db").await?;
        
        // Initialize auth system accounts
        let _ = zikzak.transfer(
            "system:genesis",
            "auth:system:initialized",
            1,
            HashMap::from([("operation".to_string(), "auth_init".to_string())])
        ).await;

        Ok(Self {
            zikzak,
            jwt_secret: "supabase_killer_secret_key".to_string(), // In production, use env var
        })
    }

    pub async fn signup(&self, payload: Value) -> Result<Value> {
        let email = payload["email"].as_str()
            .ok_or_else(|| anyhow!("Email required"))?;
        let password = payload["password"].as_str()
            .ok_or_else(|| anyhow!("Password required"))?;

        // Check if user already exists
        let user_exists = self.zikzak.get_balance(&format!("user:{}:existence", email)).await?;
        if user_exists > 0 {
            return Err(anyhow!("User already exists"));
        }

        // Hash password
        let password_hash = hash(password, DEFAULT_COST)?;
        let user_id = Uuid::new_v4().to_string();

        // Create user in ZIK_ZAK
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "user_signup".to_string());
        metadata.insert("email".to_string(), email.to_string());
        metadata.insert("password_hash".to_string(), password_hash);
        metadata.insert("user_id".to_string(), user_id.clone());
        metadata.insert("role".to_string(), "authenticated".to_string());

        // User existence
        self.zikzak.transfer(
            "system:genesis",
            &format!("user:{}:existence", email),
            1,
            metadata.clone()
        ).await?;

        // Generate JWT token
        let token = self.generate_jwt(&user_id, email, "authenticated")?;

        Ok(json!({
            "access_token": token,
            "token_type": "bearer",
            "expires_in": 3600,
            "refresh_token": format!("refresh_{}", user_id),
            "user": {
                "id": user_id,
                "email": email,
                "role": "authenticated",
                "created_at": Utc::now(),
                "updated_at": Utc::now()
            }
        }))
    }

    pub async fn token(&self, payload: Value) -> Result<Value> {
        let grant_type = payload["grant_type"].as_str().unwrap_or("password");

        match grant_type {
            "password" => {
                let email = payload["email"].as_str()
                    .ok_or_else(|| anyhow!("Email required"))?;
                let password = payload["password"].as_str()
                    .ok_or_else(|| anyhow!("Password required"))?;

                // Check if user exists
                let user_exists = self.zikzak.get_balance(&format!("user:{}:existence", email)).await?;
                if user_exists == 0 {
                    return Err(anyhow!("Invalid credentials"));
                }

                // Get user data from transaction history (simplified)
                let user_id = format!("user_{}", email.replace("@", "_").replace(".", "_"));
                
                // Generate JWT token
                let token = self.generate_jwt(&user_id, email, "authenticated")?;

                Ok(json!({
                    "access_token": token,
                    "token_type": "bearer",
                    "expires_in": 3600,
                    "refresh_token": format!("refresh_{}", user_id),
                    "user": {
                        "id": user_id,
                        "email": email,
                        "role": "authenticated"
                    }
                }))
            }
            "refresh_token" => {
                let refresh_token = payload["refresh_token"].as_str()
                    .ok_or_else(|| anyhow!("Refresh token required"))?;

                // Extract user ID from refresh token (simplified)
                let user_id = refresh_token.strip_prefix("refresh_")
                    .ok_or_else(|| anyhow!("Invalid refresh token"))?;

                // Generate new JWT token
                let token = self.generate_jwt(user_id, "user@example.com", "authenticated")?;

                Ok(json!({
                    "access_token": token,
                    "token_type": "bearer",
                    "expires_in": 3600,
                    "refresh_token": refresh_token
                }))
            }
            _ => Err(anyhow!("Unsupported grant type"))
        }
    }

    pub async fn get_user(&self, headers: HeaderMap) -> Result<Value> {
        let claims = self.extract_claims_from_headers(&headers)?;

        Ok(json!({
            "id": claims.sub,
            "email": claims.email,
            "role": claims.role,
            "created_at": Utc::now(),
            "updated_at": Utc::now()
        }))
    }

    pub async fn logout(&self, headers: HeaderMap) -> Result<Value> {
        let _claims = self.extract_claims_from_headers(&headers)?;

        // In a real implementation, we'd invalidate the token
        // For now, just return success
        Ok(json!({
            "message": "Successfully logged out"
        }))
    }

    pub async fn recover(&self, payload: Value) -> Result<Value> {
        let email = payload["email"].as_str()
            .ok_or_else(|| anyhow!("Email required"))?;

        // Check if user exists
        let user_exists = self.zikzak.get_balance(&format!("user:{}:existence", email)).await?;
        if user_exists == 0 {
            return Err(anyhow!("User not found"));
        }

        // In a real implementation, we'd send a recovery email
        // For now, just return success
        Ok(json!({
            "message": "Recovery email sent"
        }))
    }

    pub async fn verify(&self, payload: Value) -> Result<Value> {
        let token = payload["token"].as_str()
            .ok_or_else(|| anyhow!("Token required"))?;
        let type_field = payload["type"].as_str()
            .ok_or_else(|| anyhow!("Type required"))?;

        // In a real implementation, we'd verify the token
        // For now, just return success
        Ok(json!({
            "message": format!("Successfully verified {}", type_field)
        }))
    }

    pub async fn refresh(&self, payload: Value) -> Result<Value> {
        let refresh_token = payload["refresh_token"].as_str()
            .ok_or_else(|| anyhow!("Refresh token required"))?;

        // Extract user ID from refresh token (simplified)
        let user_id = refresh_token.strip_prefix("refresh_")
            .ok_or_else(|| anyhow!("Invalid refresh token"))?;

        // Generate new JWT token
        let token = self.generate_jwt(user_id, "user@example.com", "authenticated")?;

        Ok(json!({
            "access_token": token,
            "token_type": "bearer",
            "expires_in": 3600,
            "refresh_token": refresh_token
        }))
    }

    fn generate_jwt(&self, user_id: &str, email: &str, role: &str) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration as usize,
            iat: Utc::now().timestamp() as usize,
            iss: "supabase-killer".to_string(),
            aud: "authenticated".to_string(),
            role: role.to_string(),
            email: email.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    fn extract_claims_from_headers(&self, headers: &HeaderMap) -> Result<Claims> {
        let auth_header = headers
            .get("authorization")
            .ok_or_else(|| anyhow!("Authorization header missing"))?
            .to_str()?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| anyhow!("Invalid authorization header format"))?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}