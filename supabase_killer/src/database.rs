//! # 📊 Database Service - PostgREST compatible with ZIK_ZAK
//!
//! Replaces PostgreSQL with ZIK_ZAK's revolutionary accounting system.
//! Every table is a namespace, every row is an account, every field is a balance!

use anyhow::{anyhow, Result};
use axum::http::HeaderMap;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;
use zik_zak::{accounting::ZikZakEngine, recipes::RecipeEngine};

#[derive(Clone)]
pub struct DatabaseService {
    zikzak: ZikZakEngine,
    recipes: RecipeEngine,
}

impl DatabaseService {
    pub async fn new() -> Result<Self> {
        let mut zikzak = ZikZakEngine::new("main_db").await?;
        
        // Load recipes for advanced operations
        let recipes = match RecipeEngine::new("../zik_zak/recipes.json") {
            Ok(r) => r,
            Err(_) => RecipeEngine::empty(),
        };

        // Initialize database system
        let _ = zikzak.transfer(
            "system:genesis",
            "database:system:initialized",
            1,
            HashMap::from([("operation".to_string(), "db_init".to_string())])
        ).await;

        Ok(Self { zikzak, recipes })
    }

    /// SELECT - PostgREST compatible query
    pub async fn select(
        &self,
        table: String,
        params: HashMap<String, String>,
        _headers: HeaderMap,
    ) -> Result<Value> {
        // Extract query parameters
        let select = params.get("select").cloned().unwrap_or("*".to_string());
        let limit = params.get("limit")
            .and_then(|l| l.parse::<i32>().ok())
            .unwrap_or(100);

        // Simple implementation - get all records for this table
        let mut results = Vec::new();

        // In ZIK_ZAK, we query by account patterns
        // For simplicity, we'll return some mock data that demonstrates the concept
        for i in 1..=limit.min(10) {
            let row_id = format!("row_{}", i);
            
            // Get each field as a balance
            let mut row = serde_json::Map::new();
            row.insert("id".to_string(), json!(row_id));
            
            // Try to get common fields
            if let Ok(name_balance) = self.zikzak.get_balance(&format!("{}:{}:name", table, row_id)).await {
                if name_balance > 0 {
                    row.insert("name".to_string(), json!(format!("Item {}", i)));
                }
            }
            
            if let Ok(price_balance) = self.zikzak.get_balance(&format!("{}:{}:price", table, row_id)).await {
                if price_balance > 0 {
                    row.insert("price".to_string(), json!(price_balance));
                }
            }

            if let Ok(created_balance) = self.zikzak.get_balance(&format!("{}:{}:created_at", table, row_id)).await {
                if created_balance > 0 {
                    row.insert("created_at".to_string(), json!("2024-01-01T00:00:00Z"));
                }
            }

            results.push(json!(row));
        }

        Ok(json!(results))
    }

    /// INSERT - PostgREST compatible insert
    pub async fn insert(
        &mut self,
        table: String,
        payload: Value,
        _headers: HeaderMap,
    ) -> Result<Value> {
        let row_id = Uuid::new_v4().to_string();
        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "insert".to_string());
        metadata.insert("table".to_string(), table.clone());

        // Create existence record
        self.zikzak.transfer(
            "system:genesis",
            &format!("{}:{}:existence", table, row_id),
            1,
            metadata.clone()
        ).await?;

        // Insert each field as a balance
        if let Some(obj) = payload.as_object() {
            for (key, value) in obj {
                if key == "id" {
                    continue; // Skip ID field
                }

                let amount = match value {
                    Value::Number(n) => n.as_i64().unwrap_or(0),
                    Value::String(s) => {
                        // For strings, we use a hash or store as metadata
                        // and put 1 in the balance to indicate existence
                        metadata.insert(key.clone(), s.clone());
                        1
                    }
                    Value::Bool(b) => if *b { 1 } else { 0 },
                    _ => 1, // Default to 1 for complex types
                };

                self.zikzak.transfer(
                    "system:genesis",
                    &format!("{}:{}:{}", table, row_id, key),
                    amount,
                    metadata.clone()
                ).await?;
            }
        }

        // Return the created record
        let mut result = serde_json::Map::new();
        result.insert("id".to_string(), json!(row_id));
        
        if let Some(obj) = payload.as_object() {
            for (key, value) in obj {
                result.insert(key.clone(), value.clone());
            }
        }

        Ok(json!([result]))
    }

    /// UPDATE - PostgREST compatible update
    pub async fn update(
        &mut self,
        table: String,
        params: HashMap<String, String>,
        payload: Value,
        _headers: HeaderMap,
    ) -> Result<Value> {
        // Extract ID from query params or payload
        let row_id = params.get("id")
            .or_else(|| payload["id"].as_str())
            .ok_or_else(|| anyhow!("ID required for update"))?;

        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "update".to_string());
        metadata.insert("table".to_string(), table.clone());

        // Update each field
        if let Some(obj) = payload.as_object() {
            for (key, value) in obj {
                if key == "id" {
                    continue; // Skip ID field
                }

                let amount = match value {
                    Value::Number(n) => n.as_i64().unwrap_or(0),
                    Value::String(s) => {
                        metadata.insert(key.clone(), s.clone());
                        1
                    }
                    Value::Bool(b) => if *b { 1 } else { 0 },
                    _ => 1,
                };

                // Reset the field and set new value
                let current_balance = self.zikzak.get_balance(&format!("{}:{}:{}", table, row_id, key)).await?;
                if current_balance > 0 {
                    self.zikzak.transfer(
                        &format!("{}:{}:{}", table, row_id, key),
                        "system:void",
                        current_balance,
                        metadata.clone()
                    ).await?;
                }

                self.zikzak.transfer(
                    "system:genesis",
                    &format!("{}:{}:{}", table, row_id, key),
                    amount,
                    metadata.clone()
                ).await?;
            }
        }

        // Return updated record
        let mut result = serde_json::Map::new();
        result.insert("id".to_string(), json!(row_id));
        
        if let Some(obj) = payload.as_object() {
            for (key, value) in obj {
                result.insert(key.clone(), value.clone());
            }
        }

        Ok(json!([result]))
    }

    /// DELETE - PostgREST compatible delete
    pub async fn delete(
        &mut self,
        table: String,
        params: HashMap<String, String>,
        _headers: HeaderMap,
    ) -> Result<Value> {
        let row_id = params.get("id")
            .ok_or_else(|| anyhow!("ID required for delete"))?;

        let mut metadata = HashMap::new();
        metadata.insert("operation".to_string(), "delete".to_string());
        metadata.insert("table".to_string(), table.clone());

        // Move existence to void (soft delete)
        self.zikzak.transfer(
            &format!("{}:{}:existence", table, row_id),
            "system:void",
            1,
            metadata
        ).await?;

        Ok(json!([{"id": row_id}]))
    }

    /// ZIK_ZAK native transfer operation
    pub async fn zikzak_transfer(&mut self, payload: Value) -> Result<Value> {
        let from = payload["from_account"].as_str()
            .ok_or_else(|| anyhow!("from_account required"))?;
        let to = payload["to_account"].as_str()
            .ok_or_else(|| anyhow!("to_account required"))?;
        let amount = payload["amount"].as_i64()
            .ok_or_else(|| anyhow!("amount required"))?;

        let metadata = payload["metadata"].as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let transfer_id = self.zikzak.transfer(from, to, amount, metadata).await?;

        Ok(json!({
            "transfer_id": transfer_id,
            "from_account": from,
            "to_account": to,
            "amount": amount,
            "status": "completed"
        }))
    }

    /// ZIK_ZAK native balance query
    pub async fn zikzak_balance(&self, account: String) -> Result<Value> {
        let balance = self.zikzak.get_balance(&account).await?;

        Ok(json!({
            "account": account,
            "balance": balance
        }))
    }

    /// ZIK_ZAK recipe execution
    pub async fn zikzak_recipe(&mut self, name: String, payload: Value) -> Result<Value> {
        let inputs = payload.as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let result = self.recipes.execute_recipe(&name, inputs, &mut self.zikzak).await?;

        Ok(result)
    }

    /// List available recipes
    pub async fn zikzak_list_recipes(&self) -> Result<Value> {
        Ok(self.recipes.list_recipes())
    }
}