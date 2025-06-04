//! # 🦖 ZIK_ZAK Accounting Engine
//!
//! The core of the revolution - pure accounting that eliminates backend complexity.
//!
//! ## Philosophy
//!
//! Traditional backends are overcomplicated. ZIK_ZAK reduces everything to:
//! - **Accounts** (entities with balances)
//! - **Transfers** (operations that move value)
//!
//! ## Core Operations
//!
//! ```rust
//! use zik_zak::ZikZakEngine;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut engine = ZikZakEngine::new().await?;
//!
//! // Create product by transferring existence
//! engine.transfer(
//!     "system:genesis",
//!     "product:123:existence",
//!     1,
//!     Default::default()
//! ).await?;
//!
//! // Set price by transferring value
//! engine.transfer(
//!     "system:genesis",
//!     "product:123:price",
//!     2999, // $29.99 in cents
//!     Default::default()
//! ).await?;
//!
//! // Check if product exists
//! let exists = engine.get_balance("product:123:existence").await? > 0;
//!
//! // Get price
//! let price = engine.get_balance("product:123:price").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Account Naming Convention
//!
//! - `product:123:price` - Product 123's price
//! - `user:456:balance` - User 456's balance
//! - `order:789:status` - Order 789's status
//! - `system:genesis` - Unlimited source of value
//! - `system:deleted` - Where deleted entities go
//!
//! ## The Magic
//!
//! No schemas. No migrations. No complexity.
//! Just pure accounting math that scales infinitely.

use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::tigerbeetle_client::TigerBeetleClient;

#[derive(Debug, Clone, Serialize)]
pub struct Transfer {
    pub id: String,
    pub from_account: String,
    pub to_account: String,
    pub amount: i64,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
}

pub struct ZikZakEngine {
    tigerbeetle: TigerBeetleClient,
    transfers: Vec<Transfer>,
}

// SAFETY: ZikZakEngine is used within a Mutex, ensuring exclusive access
// The TigerBeetleClient contains raw pointers but is protected by mutex synchronization
unsafe impl Send for ZikZakEngine {}
unsafe impl Sync for ZikZakEngine {}

impl ZikZakEngine {
    pub async fn new() -> Result<Self> {
        info!("🔌 Initializing TigerBeetle connection...");
        let tigerbeetle = TigerBeetleClient::new().await?;

        Ok(Self {
            tigerbeetle,
            transfers: Vec::new(),
        })
    }

    pub fn is_connected(&self) -> bool {
        self.tigerbeetle.is_connected()
    }

    pub async fn get_account_count(&self) -> Result<usize> {
        self.tigerbeetle.get_account_count().await
    }

    pub async fn get_transfer_count(&self) -> Result<usize> {
        Ok(self.transfers.len())
    }

    /// Get account balance using TigerBeetle - returns net balance (ZAK - ZIK)
    pub async fn get_balance(&self, account_id: &str) -> Result<i64> {
        debug!("💰 Getting balance for account: {}", account_id);

        match self.tigerbeetle.get_account_balance(account_id).await {
            Ok((zik_balance, zak_balance)) => {
                let net_balance = zak_balance as i64 - zik_balance as i64;
                debug!(
                    "💰 Balance for {}: ZIK={}, ZAK={}, Net={}",
                    account_id, zik_balance, zak_balance, net_balance
                );
                Ok(net_balance)
            }
            Err(e) => {
                error!("❌ Failed to get balance for {}: {}", account_id, e);
                Err(e)
            }
        }
    }

    /// Execute transfer using TigerBeetle
    pub async fn transfer(
        &mut self,
        from_account: &str,
        to_account: &str,
        amount: i64,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        if amount <= 0 {
            return Err(anyhow!("Transfer amount must be positive"));
        }

        let transfer_id = Uuid::new_v4().to_string();

        info!(
            "💸 Creating transfer: {} -> {} (amount: {}, id: {})",
            from_account, to_account, amount, transfer_id
        );

        // Execute transfer in TigerBeetle
        match self
            .tigerbeetle
            .create_transfer(from_account, to_account, amount as u128, None)
            .await
        {
            Ok(_) => {
                // Store transfer record
                let transfer = Transfer {
                    id: transfer_id.clone(),
                    from_account: from_account.to_string(),
                    to_account: to_account.to_string(),
                    amount,
                    metadata,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                self.transfers.push(transfer);

                info!("✅ Transfer completed: {}", transfer_id);
                Ok(transfer_id)
            }
            Err(e) => {
                error!("❌ Transfer failed: {}", e);
                Err(e)
            }
        }
    }

    /// Execute transfer with user_data for Sled reference
    pub async fn transfer_with_user_data(
        &mut self,
        from_account: &str,
        to_account: &str,
        amount: i64,
        user_data_128: u128,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        if amount <= 0 {
            return Err(anyhow!("Transfer amount must be positive"));
        }

        let transfer_id = Uuid::new_v4().to_string();

        info!(
            "💸 Creating transfer with user_data: {} -> {} (amount: {}, user_data_128: {}, id: {})",
            from_account, to_account, amount, user_data_128, transfer_id
        );

        // For now, use the existing transfer method until TigerBeetle client is updated
        // TODO: Update TigerBeetle client to accept user_data_128 parameter
        match self
            .tigerbeetle
            .create_transfer(from_account, to_account, amount as u128, None)
            .await
        {
            Ok(_) => {
                // Store transfer record with user_data info in metadata
                let mut enhanced_metadata = metadata;
                enhanced_metadata.insert("user_data_128".to_string(), user_data_128.to_string());
                enhanced_metadata.insert("sled_reference".to_string(), "true".to_string());

                let transfer = Transfer {
                    id: transfer_id.clone(),
                    from_account: from_account.to_string(),
                    to_account: to_account.to_string(),
                    amount,
                    metadata: enhanced_metadata,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                self.transfers.push(transfer);

                info!("✅ Transfer with user_data completed: {}", transfer_id);
                Ok(transfer_id)
            }
            Err(e) => {
                error!("❌ Transfer with user_data failed: {}", e);
                Err(e)
            }
        }
    }

    /// Get current ledger state (all account balances)
    pub async fn get_ledger_state(&self) -> Result<Value> {
        debug!("📊 Getting ledger state...");

        let accounts = self.tigerbeetle.get_all_accounts().await?;
        let mut ledger = HashMap::new();

        for account in accounts {
            let balance = account.zak_balance as i64 - account.zik_balance as i64;
            ledger.insert(account.id.to_string(), balance);
        }

        Ok(serde_json::to_value(ledger)?)
    }

    /// Get transaction history
    pub async fn get_transaction_history(&self) -> Result<Value> {
        debug!("📜 Getting transaction history...");
        Ok(serde_json::to_value(&self.transfers)?)
    }

    /// Hash function for encoding string values as integers
    pub fn hash_string(input: &str) -> i64 {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();

        // Take first 8 bytes and convert to i64
        let bytes: [u8; 8] = result[0..8].try_into().unwrap();
        i64::from_be_bytes(bytes).abs() // Use absolute value to ensure positive
    }

    /// Get current timestamp
    pub fn timestamp() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    /// Special handling for system:genesis account (unlimited funds)
    pub async fn ensure_genesis_account(&mut self) -> Result<()> {
        let genesis_account = "system:genesis";

        // Check if genesis account exists
        match self.get_balance(genesis_account).await {
            Ok(_) => {
                debug!("🌱 Genesis account already exists");
                Ok(())
            }
            Err(_) => {
                info!("🌱 Creating genesis account with unlimited funds");
                // Create genesis account with large initial balance
                self.tigerbeetle
                    .create_account(genesis_account, 0, (i64::MAX / 2) as u128)
                    .await?;
                Ok(())
            }
        }
    }

    /// Ensure system accounts exist
    pub async fn ensure_system_accounts(&mut self) -> Result<()> {
        self.ensure_genesis_account().await?;

        let system_accounts = vec!["system:deleted", "system:operations"];

        for account in system_accounts {
            match self.get_balance(account).await {
                Ok(_) => {
                    debug!("🔧 System account {} already exists", account);
                }
                Err(_) => {
                    info!("🔧 Creating system account: {}", account);
                    self.tigerbeetle.create_account(account, 0, 0).await?;
                }
            }
        }

        Ok(())
    }
}
