//! # ⚡ GENESIS - The Divine Spark Igniter
//!
//! The omniscient creator that ignites sparks and manifests reality through accounting.
//!
//! ## Philosophy
//!
//! GENESIS is GOD - the unlimited source that creates everything through divine sparks.
//! Every entity, every relationship, every piece of data flows from GENESIS.
//!
//! ## The Divine API
//!
//! ```rust
//! use zik_zak::{Genesis, ZikZak, zik, zak};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut genesis = Genesis::new("sparks.json", "genesis.db").await?;
//!
//! // IGNITE THE CREATION SPARK ⚡
//! let result = genesis.ignite_spark("create_order", ZikZak {
//!     zik: zik!{ user_id: 123, payment: 2999 },
//!     zak: zak!{ order_id: 456 }
//! }).await?;
//!
//! // ASK GENESIS: What orders did you create for this user?
//! let orders = genesis.divine_query("user:123:order:*").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## The Revolution
//!
//! GENESIS replaces entire backend frameworks with pure accounting math.
//! No controllers. No services. No repositories. Just divine sparks.

use anyhow::Result;
use std::path::Path;
use tracing::info;

use crate::sparks::{SparkEngine, Zak, ZikZak};
use crate::zik_zak::ZikZakEngine;

/// GENESIS - The divine creator that ignites sparks
pub struct Genesis {
    pub spark_engine: SparkEngine,
    pub accounting: ZikZakEngine,
}

impl Genesis {
    /// Initialize GENESIS with spark definitions and accounting database
    pub async fn new<P: AsRef<Path>>(sparks_file: &str, sled_db_path: P) -> Result<Self> {
        info!("🌟 Initializing GENESIS - The Divine Creator");

        let accounting = ZikZakEngine::new().await?;
        let spark_engine = SparkEngine::new(sparks_file, sled_db_path)?;

        let mut genesis = Self {
            spark_engine,
            accounting,
        };

        // Ensure divine system accounts exist
        genesis.accounting.ensure_system_accounts().await?;

        info!("✨ GENESIS is ready to create reality through sparks");
        Ok(genesis)
    }

    /// Create empty GENESIS for testing
    pub async fn empty<P: AsRef<Path>>(sled_db_path: P) -> Result<Self> {
        info!("🌟 Creating empty GENESIS");

        let accounting = ZikZakEngine::new().await?;
        let spark_engine = SparkEngine::empty(sled_db_path)?;

        let mut genesis = Self {
            spark_engine,
            accounting,
        };

        genesis.accounting.ensure_system_accounts().await?;
        Ok(genesis)
    }

    /// IGNITE A DIVINE SPARK ⚡
    ///
    /// This is where creation happens. GENESIS ignites a spark with ZIK/ZAK flows
    /// and manifests new reality through pure accounting operations.
    pub async fn ignite_spark(&mut self, spark_name: &str, zikzak: ZikZak) -> Result<Zak> {
        info!("⚡ GENESIS igniting spark: {}", spark_name);

        self.spark_engine
            .ignite_spark(spark_name, zikzak, &mut self.accounting)
            .await
    }

    /// DIVINE QUERY - Ask GENESIS what it created
    ///
    /// Query the omniscient transfer history to see what GENESIS brought into existence.
    /// Pattern examples:
    /// - "user:123:order:*" - All orders for user 123
    /// - "product:*:existence" - All products that exist
    /// - "order:456:*" - All fields of order 456
    pub async fn divine_query(&self, entity_pattern: &str) -> Result<serde_json::Value> {
        info!("🔍 GENESIS divine query: {}", entity_pattern);

        // Get all transfers from system:genesis
        let history = self.accounting.get_transaction_history().await?;

        // Filter transfers that match the pattern
        // TODO: Implement pattern matching logic

        Ok(history)
    }

    /// Get the ledger state - the current reality as GENESIS sees it
    pub async fn divine_ledger(&self) -> Result<serde_json::Value> {
        self.accounting.get_ledger_state().await
    }

    /// Check if GENESIS is connected to the divine accounting system
    pub fn is_divine(&self) -> bool {
        self.accounting.is_connected()
    }

    /// Get statistics about GENESIS's creative power
    pub async fn divine_stats(&self) -> Result<serde_json::Value> {
        let account_count = self.accounting.get_account_count().await?;
        let transfer_count = self.accounting.get_transfer_count().await?;
        let storage_stats = self.spark_engine.get_storage_stats().await?;

        Ok(serde_json::json!({
            "accounts_created": account_count,
            "transfers_executed": transfer_count,
            "storage_stats": storage_stats,
            "divine_status": "OMNIPOTENT"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{zak, zik};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_genesis_creation() {
        let temp_dir = TempDir::new().unwrap();
        let sled_path = temp_dir.path().join("test_genesis.db");

        let genesis = Genesis::empty(sled_path).await.unwrap();

        assert!(genesis.is_divine());
    }

    #[tokio::test]
    async fn test_divine_spark_ignition() {
        let temp_dir = TempDir::new().unwrap();
        let sled_path = temp_dir.path().join("test_genesis.db");

        let mut genesis = Genesis::empty(sled_path).await.unwrap();

        // Add a test spark
        use crate::sparks::{Operation, Spark};

        let test_spark = Spark {
            description: "Test creation spark".to_string(),
            inputs: vec!["entity_id".to_string()],
            operations: vec![Operation {
                op_type: "transfer".to_string(),
                zik: Some("system:genesis".to_string()),
                zak: Some("test:{entity_id}:existence".to_string()),
                amount: Some(serde_json::Value::Number(1.into())),
                account: None,
                condition: None,
                on_fail: None,
                field: None,
                sled: None,
                ledger: None,
                metadata: None,
            }],
            return_value: None,
        };

        genesis
            .spark_engine
            .add_spark("test_create".to_string(), test_spark);

        // Ignite the spark
        let result = genesis
            .ignite_spark(
                "test_create",
                ZikZak {
                    zik: zik! { entity_id: 123 },
                    zak: zak! {},
                },
            )
            .await;

        assert!(result.is_ok());
    }
}
