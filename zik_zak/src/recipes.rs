//! # 🍳 ZIK_ZAK Recipe Engine
//!
//! JSON-driven operations that eliminate the need for code.
//!
//! ## Philosophy
//!
//! Why write code when you can write recipes? Traditional backends require:
//! - Controllers for each entity
//! - Services for business logic
//! - Repositories for data access
//! - Tests for everything
//!
//! ZIK_ZAK recipes replace ALL of that with JSON:
//!
//! ```json
//! {
//!   "create_product": {
//!     "description": "Create a new product",
//!     "inputs": ["id", "name", "price"],
//!     "operations": [
//!       {
//!         "type": "transfer",
//!         "from": "system:genesis",
//!         "to": "product:{id}:existence",
//!         "amount": 1
//!       },
//!       {
//!         "type": "transfer",
//!         "from": "system:genesis",
//!         "to": "product:{id}:price",
//!         "amount": "{price}"
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! ## Recipe Operations
//!
//! - `transfer` - Move value between accounts
//! - `balance` - Check account balance with conditions
//! - `get_metadata` - Extract transaction metadata
//!
//! ## The Revolution
//!
//! No more code. No more deployments. No more complexity.
//! Just edit JSON and watch your backend evolve in real-time.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info};

use crate::accounting::ZikZakEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub description: String,
    pub inputs: Vec<String>,
    pub operations: Vec<Operation>,
    #[serde(rename = "return")]
    pub return_value: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub account: Option<String>,
    pub amount: Option<Value>,
    pub condition: Option<String>,
    pub on_fail: Option<String>,
    pub field: Option<String>,
    pub store_as: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeDefinition {
    pub schema_version: String,
    pub title: String,
    pub description: String,
    pub primitives: HashMap<String, String>,
    pub entities: Value,
    pub recipes: HashMap<String, Recipe>,
}

#[derive(Clone)]
pub struct RecipeEngine {
    recipes: HashMap<String, Recipe>,
}

// SAFETY: RecipeEngine only contains HashMap<String, Recipe> which are both Send + Sync
// Recipe contains only basic types (String, Vec, HashMap) which are Send + Sync
unsafe impl Send for RecipeEngine {}
unsafe impl Sync for RecipeEngine {}

impl RecipeEngine {
    pub fn new(recipes_file: &str) -> Result<Self> {
        info!("🍳 Loading recipes from: {}", recipes_file);

        let recipes_content = fs::read_to_string(recipes_file)
            .map_err(|e| anyhow!("Failed to read recipes file: {}", e))?;

        let recipe_def: RecipeDefinition = serde_json::from_str(&recipes_content)
            .map_err(|e| anyhow!("Failed to parse recipes JSON: {}", e))?;

        info!("✅ Loaded {} recipes", recipe_def.recipes.len());

        Ok(Self {
            recipes: recipe_def.recipes,
        })
    }

    pub fn empty() -> Self {
        info!("🍳 Creating empty recipe engine");
        Self {
            recipes: HashMap::new(),
        }
    }

    pub fn list_recipes(&self) -> Value {
        let mut recipe_list = HashMap::new();

        for (name, recipe) in &self.recipes {
            recipe_list.insert(
                name,
                json!({
                    "description": recipe.description,
                    "inputs": recipe.inputs,
                    "operations_count": recipe.operations.len()
                }),
            );
        }

        serde_json::to_value(recipe_list).unwrap()
    }

    pub async fn execute_recipe(
        &self,
        recipe_name: &str,
        inputs: HashMap<String, Value>,
        accounting: &mut ZikZakEngine,
    ) -> Result<Value> {
        let recipe = self
            .recipes
            .get(recipe_name)
            .ok_or_else(|| anyhow!("Recipe not found: {}", recipe_name))?;

        info!("🍳 Executing recipe: {}", recipe_name);
        debug!("📥 Recipe inputs: {:?}", inputs);

        let mut stored_values = HashMap::new();

        for (i, operation) in recipe.operations.iter().enumerate() {
            debug!("🔄 Executing operation {}: {:?}", i + 1, operation.op_type);

            match self
                .execute_operation(operation, &inputs, &stored_values, accounting)
                .await
            {
                Ok(result) => {
                    if let Some(store_as) = &operation.store_as {
                        stored_values.insert(store_as.clone(), result);
                    }
                }
                Err(e) => {
                    if let Some(on_fail) = &operation.on_fail {
                        if on_fail.starts_with("return") {
                            return Ok(Value::Null);
                        } else if on_fail.starts_with("throw") {
                            return Err(e);
                        }
                    }
                    return Err(e);
                }
            }
        }

        // Build return value
        if let Some(return_template) = &recipe.return_value {
            let mut result = HashMap::new();

            for (key, template) in return_template {
                let value = self.interpolate(template, &inputs, &stored_values);
                result.insert(key.clone(), value);
            }

            Ok(serde_json::to_value(result)?)
        } else {
            Ok(serde_json::to_value(stored_values)?)
        }
    }

    async fn execute_operation(
        &self,
        operation: &Operation,
        inputs: &HashMap<String, Value>,
        stored: &HashMap<String, Value>,
        accounting: &mut ZikZakEngine,
    ) -> Result<Value> {
        match operation.op_type.as_str() {
            "transfer" => {
                let from = self.interpolate(
                    operation
                        .from
                        .as_ref()
                        .ok_or(anyhow!("Missing 'from' field"))?,
                    inputs,
                    stored,
                );
                let to = self.interpolate(
                    operation.to.as_ref().ok_or(anyhow!("Missing 'to' field"))?,
                    inputs,
                    stored,
                );
                let amount = self.evaluate_amount(
                    operation
                        .amount
                        .as_ref()
                        .ok_or(anyhow!("Missing 'amount' field"))?,
                    inputs,
                    stored,
                )?;

                let metadata = operation
                    .metadata
                    .as_ref()
                    .map(|m| self.interpolate_metadata(m, inputs, stored))
                    .unwrap_or_default();

                debug!("Executing transfer: {} -> {} ({})", from, to, amount);

                // Actually execute the transfer
                let transfer_id = accounting.transfer(&from, &to, amount, metadata).await?;

                Ok(Value::String(transfer_id))
            }
            "balance" => {
                let account = self.interpolate(
                    operation
                        .account
                        .as_ref()
                        .ok_or(anyhow!("Missing 'account' field"))?,
                    inputs,
                    stored,
                );

                let balance = accounting.get_balance(&account).await?;

                if let Some(condition) = &operation.condition {
                    if condition == "> 0" && balance <= 0 {
                        return Err(anyhow!(
                            "Balance condition failed: {} = {}",
                            account,
                            balance
                        ));
                    }
                }

                Ok(Value::Number(serde_json::Number::from(balance)))
            }
            "get_metadata" => {
                let account = self.interpolate(
                    operation
                        .account
                        .as_ref()
                        .ok_or(anyhow!("Missing 'account' field"))?,
                    inputs,
                    stored,
                );
                let field = operation
                    .field
                    .as_ref()
                    .ok_or(anyhow!("Missing 'field' field"))?;

                // Get transaction history and find metadata for this account
                let _history = accounting.get_transaction_history().await?;

                // For simplicity, return the field name for now
                // In a real implementation, we'd parse the transaction history
                Ok(Value::String(format!("{}_{}", account, field)))
            }
            _ => Err(anyhow!("Unknown operation type: {}", operation.op_type)),
        }
    }

    fn interpolate(
        &self,
        template: &str,
        inputs: &HashMap<String, Value>,
        stored: &HashMap<String, Value>,
    ) -> String {
        let mut result = template.to_string();

        // Replace input variables
        for (key, value) in inputs {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        // Replace stored variables
        for (key, value) in stored {
            let placeholder = format!("{{{}}}", key);
            let replacement = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }

        result
    }

    fn interpolate_metadata(
        &self,
        metadata: &HashMap<String, String>,
        inputs: &HashMap<String, Value>,
        stored: &HashMap<String, Value>,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();

        for (key, value) in metadata {
            result.insert(key.clone(), self.interpolate(value, inputs, stored));
        }

        result
    }

    fn evaluate_amount(
        &self,
        amount_expr: &Value,
        inputs: &HashMap<String, Value>,
        stored: &HashMap<String, Value>,
    ) -> Result<i64> {
        match amount_expr {
            Value::Number(n) => Ok(n.as_i64().unwrap_or(0)),
            Value::String(s) => {
                let interpolated = self.interpolate(s, inputs, stored);

                // Handle special functions
                if interpolated.starts_with("hash(") && interpolated.ends_with(")") {
                    let value = &interpolated[5..interpolated.len() - 1];
                    Ok(ZikZakEngine::hash_string(value))
                } else if interpolated == "timestamp()" {
                    Ok(ZikZakEngine::timestamp())
                } else {
                    // Try to parse as number
                    interpolated
                        .parse::<i64>()
                        .map_err(|_| anyhow!("Cannot evaluate amount: {}", interpolated))
                }
            }
            _ => Err(anyhow!("Invalid amount type")),
        }
    }
}
