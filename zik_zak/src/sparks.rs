//! # ⚡ ZIK_ZAK Spark Engine
//!
//! JSON-driven sparks that eliminate the need for code.
//!
//! ## Philosophy
//!
//! Why write code when you can ignite sparks? Traditional backends require:
//! - Controllers for each entity
//! - Services for business logic
//! - Repositories for data access
//! - Tests for everything
//!
//! ZIK_ZAK sparks replace ALL of that with JSON:
//!
//! ```json
//! {
//!   "create_product": {
//!     "description": "Spark that births products into existence",
//!     "inputs": ["id", "name", "price"],
//!     "operations": [
//!       {
//!         "type": "transfer",
//!         "zik": "system:genesis",
//!         "zak": "product:{id}:existence",
//!         "amount": 1
//!       },
//!       {
//!         "type": "transfer",
//!         "zik": "system:genesis",
//!         "zak": "product:{id}:price",
//!         "amount": "{price}"
//!       },
//!       {
//!         "type": "transfer",
//!         "zik": "system:genesis",
//!         "zak": "product:{id}:name",
//!         "amount": "{name}",
//!         "sled": true
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! ## Spark Operations
//!
//! - `transfer` - Move value between accounts (ZIK→ZAK flow)
//! - `balance` - Check account balance with conditions
//! - `get_metadata` - Extract transaction metadata
//!
//! ## Storage Strategy
//!
//! - **Numbers, booleans, enums** → TigerBeetle balance only
//! - **Text/varchar** → Sled storage with `user_data_128` as key
//!
//! ## Data Types
//!
//! | Type    | Storage       | Example                           |
//! |---------|---------------|-----------------------------------|
//! | Number  | TB balance    | `product:123:price = 2999`       |
//! | Boolean | TB balance    | `user:456:active = 1`            |
//! | Enum    | TB balance    | `order:789:status = 2` (shipped) |
//! | Text    | Sled + TB ref | `account:234:username = john_doe` |
//!
//! ## The Revolution
//!
//! No more code. No more deployments. No more complexity.
//! Just ignite sparks and watch your backend evolve in real-time.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use std::fs;
use std::path::Path;
use tracing::{debug, info};
use xxhash_rust::xxh3::xxh3_64;

use crate::sled::SledVarCharStore;
use crate::zik_zak::ZikZakEngine;

/// ZIK flow - what flows OUT (source, give, debit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zik(pub HashMap<String, Value>);

/// ZAK flow - what flows IN (destination, receive, credit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zak(pub HashMap<String, Value>);

/// ZIK_ZAK flow combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZikZak {
    pub zik: Zik,
    pub zak: Zak,
}

/// Convenience macros for creating ZIK/ZAK flows
#[macro_export]
macro_rules! zik {
    ($($key:ident: $value:expr),*) => {
        $crate::Zik(std::collections::HashMap::from([
            $((stringify!($key).to_string(), serde_json::to_value($value).unwrap())),*
        ]))
    };
}

#[macro_export]
macro_rules! zak {
    ($($key:ident: $value:expr),*) => {
        $crate::Zak(std::collections::HashMap::from([
            $((stringify!($key).to_string(), serde_json::to_value($value).unwrap())),*
        ]))
    };
}

impl Zik {
    pub fn new(data: HashMap<String, Value>) -> Self {
        Self(data)
    }

    pub fn into_map(self) -> HashMap<String, Value> {
        self.0
    }
}

impl Zak {
    pub fn new(data: HashMap<String, Value>) -> Self {
        Self(data)
    }

    pub fn into_map(self) -> HashMap<String, Value> {
        self.0
    }

    pub fn from_result(result: Value) -> Self {
        match result {
            Value::Object(map) => {
                let converted: HashMap<String, Value> = map.into_iter().collect();
                Self(converted)
            }
            _ => Self(HashMap::new()),
        }
    }
}

impl ZikZak {
    pub fn new(zik: Zik, zak: Zak) -> Self {
        Self { zik, zak }
    }

    pub fn inputs(&self) -> HashMap<String, Value> {
        let mut inputs = HashMap::new();
        inputs.extend(self.zik.0.clone());
        inputs.extend(self.zak.0.clone());
        inputs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spark {
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
    pub zik: Option<String>, // ZIK account (OUT)
    pub zak: Option<String>, // ZAK account (IN)
    pub account: Option<String>,
    pub amount: Option<Value>,
    pub condition: Option<String>,
    pub on_fail: Option<String>,
    pub field: Option<String>,
    pub sled: Option<bool>,  // true = store text in Sled
    pub ledger: Option<u32>, // TigerBeetle ledger ID (defaults to 1)
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkDefinition {
    pub schema_version: String,
    pub title: String,
    pub description: String,
    pub primitives: HashMap<String, String>,
    pub entities: Value,
    pub sparks: HashMap<String, Spark>,
}

pub struct SparkEngine {
    sparks: HashMap<String, Spark>,
    sled_store: SledVarCharStore,
}

// SAFETY: SparkEngine only contains HashMap<String, Spark> and SledVarCharStore
// Both are safe to send across threads when properly synchronized
unsafe impl Send for SparkEngine {}
unsafe impl Sync for SparkEngine {}

impl SparkEngine {
    pub fn new<P: AsRef<Path>>(sparks_file: &str, sled_db_path: P) -> Result<Self> {
        info!("⚡ Loading sparks from: {}", sparks_file);

        let sparks_content = fs::read_to_string(sparks_file)
            .map_err(|e| anyhow!("Failed to read sparks file: {}", e))?;

        let spark_def: SparkDefinition = serde_json::from_str(&sparks_content)
            .map_err(|e| anyhow!("Failed to parse sparks JSON: {}", e))?;

        let sled_store = SledVarCharStore::new(sled_db_path)?;

        info!("✅ Loaded {} sparks", spark_def.sparks.len());

        Ok(Self {
            sparks: spark_def.sparks,
            sled_store,
        })
    }

    pub fn empty<P: AsRef<Path>>(sled_db_path: P) -> Result<Self> {
        info!("⚡ Creating empty spark engine");
        let sled_store = SledVarCharStore::new(sled_db_path)?;

        Ok(Self {
            sparks: HashMap::new(),
            sled_store,
        })
    }

    pub fn list_sparks(&self) -> Value {
        let mut spark_list = HashMap::new();

        for (name, spark) in &self.sparks {
            spark_list.insert(
                name,
                json!({
                    "description": spark.description,
                    "inputs": spark.inputs,
                    "operations_count": spark.operations.len()
                }),
            );
        }

        serde_json::to_value(spark_list).unwrap()
    }

    pub async fn ignite_spark(
        &self,
        spark_name: &str,
        zikzak: ZikZak,
        accounting: &mut ZikZakEngine,
    ) -> Result<Zak> {
        let spark = self
            .sparks
            .get(spark_name)
            .ok_or_else(|| anyhow!("Spark not found: {}", spark_name))?;

        info!("⚡ Igniting spark: {}", spark_name);
        let inputs = zikzak.inputs();
        debug!("📥 Spark inputs: {:?}", inputs);

        let mut stored_values = HashMap::new();

        for (i, operation) in spark.operations.iter().enumerate() {
            debug!("🔄 Executing operation {}: {:?}", i + 1, operation.op_type);

            match self
                .execute_operation(operation, &inputs, &stored_values, accounting)
                .await
            {
                Ok(result) => {
                    // Store result with operation index as key
                    stored_values.insert(format!("op_{}", i), result);
                }
                Err(e) => {
                    if let Some(on_fail) = &operation.on_fail {
                        if on_fail.starts_with("return") {
                            return Ok(Zak::new(HashMap::new()));
                        } else if on_fail.starts_with("throw") {
                            return Err(e);
                        }
                    }
                    return Err(e);
                }
            }
        }

        // Build return value
        if let Some(return_template) = &spark.return_value {
            let mut result = HashMap::new();

            for (key, template) in return_template {
                let value = self.interpolate(template, &inputs, &stored_values);
                result.insert(key.clone(), serde_json::to_value(value)?);
            }

            Ok(Zak::new(result))
        } else {
            Ok(Zak::new(stored_values))
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
                let zik_account = self.interpolate(
                    operation
                        .zik
                        .as_ref()
                        .ok_or(anyhow!("Missing 'zik' field"))?,
                    inputs,
                    stored,
                );
                let zak_account = self.interpolate(
                    operation
                        .zak
                        .as_ref()
                        .ok_or(anyhow!("Missing 'zak' field"))?,
                    inputs,
                    stored,
                );

                let is_sled = operation.sled.unwrap_or(false);
                let ledger_id = operation.ledger.unwrap_or(1);

                let metadata = operation
                    .metadata
                    .as_ref()
                    .map(|m| self.interpolate_metadata(m, inputs, stored))
                    .unwrap_or_default();

                if is_sled {
                    // Text storage: Store in Sled and create TigerBeetle reference
                    let value = self.interpolate(
                        &operation
                            .amount
                            .as_ref()
                            .ok_or(anyhow!("Missing 'amount' field for text transfer"))?
                            .to_string(),
                        inputs,
                        stored,
                    );

                    debug!(
                        "Executing text transfer: {} -> {} ({})",
                        zik_account, zak_account, value
                    );

                    // Generate Sled key using account name hash
                    let sled_key = self.generate_sled_key(&zak_account);

                    // Store text in Sled
                    let mut sled_metadata = metadata.clone();
                    sled_metadata.insert("ledger_id".to_string(), ledger_id.to_string());
                    sled_metadata.insert("zik_account".to_string(), zik_account.clone());
                    sled_metadata.insert("zak_account".to_string(), zak_account.clone());
                    sled_metadata.insert("storage_type".to_string(), "sled".to_string());

                    let _record_key = self
                        .sled_store
                        .store_varchar(&zak_account, "value", &value, "text/plain", sled_metadata)
                        .await?;

                    // Create TigerBeetle reference with Sled key in user_data_128
                    let transfer_id = accounting
                        .transfer_with_user_data(&zik_account, &zak_account, 1, sled_key, metadata)
                        .await?;

                    Ok(Value::String(transfer_id))
                } else {
                    // Numeric/boolean/enum storage: Direct TigerBeetle
                    let amount = self.evaluate_amount(
                        operation
                            .amount
                            .as_ref()
                            .ok_or(anyhow!("Missing 'amount' field"))?,
                        inputs,
                        stored,
                    )?;

                    debug!(
                        "Executing numeric transfer: {} -> {} ({}) on ledger {}",
                        zik_account, zak_account, amount, ledger_id
                    );

                    let transfer_id = accounting
                        .transfer(&zik_account, &zak_account, amount, metadata)
                        .await?;
                    Ok(Value::String(transfer_id))
                }
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

                let is_sled = operation.sled.unwrap_or(false);

                if is_sled {
                    // Text balance: Get from Sled using TigerBeetle user_data_128 as key
                    let tb_balance = accounting.get_balance(&account).await?;

                    if tb_balance > 0 {
                        // TigerBeetle has reference, get user_data_128 for Sled key
                        // For now, try direct Sled lookup with account name
                        match self.sled_store.get_varchar(&account, "value").await? {
                            Some(content) => Ok(Value::String(content)),
                            None => Ok(Value::Null),
                        }
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    // Numeric/boolean/enum balance: Direct TigerBeetle
                    let balance = accounting.get_balance(&account).await?;

                    if let Some(condition) = &operation.condition {
                        if condition == "> 0" && balance <= 0 {
                            return Err(anyhow!(
                                "Balance condition failed: {} = {}",
                                account,
                                balance
                            ));
                        }
                        if condition.starts_with("== ") {
                            let expected: i64 = condition[3..]
                                .parse()
                                .map_err(|_| anyhow!("Invalid balance condition: {}", condition))?;
                            if balance != expected {
                                return Err(anyhow!(
                                    "Balance condition failed: {} = {} (expected {})",
                                    account,
                                    balance,
                                    expected
                                ));
                            }
                        }
                        if condition.starts_with(">= ") {
                            let min_balance: i64 = condition[3..]
                                .parse()
                                .map_err(|_| anyhow!("Invalid balance condition: {}", condition))?;
                            if balance < min_balance {
                                return Err(anyhow!(
                                    "Balance condition failed: {} = {} (expected >= {})",
                                    account,
                                    balance,
                                    min_balance
                                ));
                            }
                        }
                    }

                    Ok(Value::Number(serde_json::Number::from(balance)))
                }
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

                debug!("Getting metadata for: {}:{}", account, field);

                // Get transaction history and find metadata for this account
                let _history = accounting.get_transaction_history().await?;

                // For simplicity, return the field name for now
                // In a real implementation, we'd parse the transaction history
                Ok(Value::String(format!("{}_{}", account, field)))
            }
            _ => Err(anyhow!("Unknown operation type: {}", operation.op_type)),
        }
    }

    /// Generate Sled key from account name using xxHash
    fn generate_sled_key(&self, account: &str) -> u128 {
        let hash = xxh3_64(account.as_bytes());
        // Convert u64 to u128 for compatibility
        hash as u128
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
                Value::Bool(b) => b.to_string(),
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
                Value::Bool(b) => b.to_string(),
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
            Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
            Value::String(s) => {
                let interpolated = self.interpolate(s, inputs, stored);

                // Handle special functions
                if interpolated.starts_with("hash(") && interpolated.ends_with(")") {
                    let value = &interpolated[5..interpolated.len() - 1];
                    Ok(ZikZakEngine::hash_string(value))
                } else if interpolated == "timestamp()" {
                    Ok(ZikZakEngine::timestamp())
                } else if interpolated == "true" {
                    Ok(1)
                } else if interpolated == "false" {
                    Ok(0)
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

    /// Add or update a spark at runtime
    pub fn add_spark(&mut self, name: String, spark: Spark) {
        info!("➕ Adding spark: {}", name);
        self.sparks.insert(name, spark);
    }

    /// Remove a spark at runtime
    pub fn remove_spark(&mut self, name: &str) -> Option<Spark> {
        info!("➖ Removing spark: {}", name);
        self.sparks.remove(name)
    }

    /// Get spark details
    pub fn get_spark(&self, name: &str) -> Option<&Spark> {
        self.sparks.get(name)
    }

    /// Get Sled storage statistics
    pub async fn get_storage_stats(&self) -> Result<serde_json::Value> {
        let stats = self.sled_store.get_stats().await?;
        Ok(serde_json::to_value(stats)?)
    }
}
