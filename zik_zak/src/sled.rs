//! # 🦖 ZIK_ZAK SLED VARCHAR STORAGE
//!
//! The final piece of the puzzle - SLED for persistent varchar fields.
//!
//! ## Philosophy
//!
//! TigerBeetle handles the numeric accounting perfectly.
//! SLED handles the varchar/text data persistently.
//! Together = Complete backend replacement.
//!
//! ## Architecture
//!
//! ```
//! ┌─────────────────┐    ┌─────────────────┐
//! │   TigerBeetle   │    │      SLED       │
//! │   (Numbers)     │    │   (VarChar)     │
//! │                 │    │                 │
//! │ account:balance │    │ account:name    │
//! │ product:price   │    │ product:title   │
//! │ order:status    │    │ order:notes     │
//! └─────────────────┘    └─────────────────┘
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sled::{Db, Tree};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarCharRecord {
    pub account_id: String,
    pub field_name: String,
    pub content: String,
    pub content_type: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub metadata: HashMap<String, String>,
}

/// 🗄️ SLED-based VARCHAR storage engine
pub struct SledVarCharStore {
    db: Db,
    records_tree: Tree,
    accounts_tree: Tree,
    content_hash_tree: Tree,
}

impl SledVarCharStore {
    /// Initialize SLED database for varchar storage
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        info!(
            "🗄️ Initializing SLED VARCHAR store at: {:?}",
            db_path.as_ref()
        );

        let db = sled::open(db_path)?;

        // Create trees for different access patterns
        let records_tree = db.open_tree("varchar_records")?;
        let accounts_tree = db.open_tree("account_fields")?;
        let content_hash_tree = db.open_tree("content_hash_lookup")?;

        Ok(Self {
            db,
            records_tree,
            accounts_tree,
            content_hash_tree,
        })
    }

    /// Store varchar field for an account
    pub async fn store_varchar(
        &self,
        account_id: &str,
        field_name: &str,
        content: &str,
        content_type: &str,
        metadata: HashMap<String, String>,
    ) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let record = VarCharRecord {
            account_id: account_id.to_string(),
            field_name: field_name.to_string(),
            content: content.to_string(),
            content_type: content_type.to_string(),
            created_at: now,
            updated_at: now,
            metadata,
        };

        // Primary key: account_id:field_name
        let key = format!("{}:{}", account_id, field_name);
        let value = serde_json::to_vec(&record)?;

        // Store in main records tree
        self.records_tree.insert(&key, value)?;

        // Index by account for fast account-based queries
        let account_key = format!("account:{}", account_id);
        let mut account_fields: Vec<String> = self
            .accounts_tree
            .get(&account_key)?
            .map(|v| serde_json::from_slice(&v).unwrap_or_default())
            .unwrap_or_default();

        if !account_fields.contains(&field_name.to_string()) {
            account_fields.push(field_name.to_string());
            self.accounts_tree
                .insert(&account_key, serde_json::to_vec(&account_fields)?)?;
        }

        // Index by content hash for deduplication/search
        let content_hash = Self::hash_content(content);
        let hash_key = format!("hash:{}", content_hash);
        let mut hash_records: Vec<String> = self
            .content_hash_tree
            .get(&hash_key)?
            .map(|v| serde_json::from_slice(&v).unwrap_or_default())
            .unwrap_or_default();

        if !hash_records.contains(&key) {
            hash_records.push(key.clone());
            self.content_hash_tree
                .insert(&hash_key, serde_json::to_vec(&hash_records)?)?;
        }

        // Ensure durability
        self.db.flush()?;

        debug!("📝 Stored varchar: {} = {}", key, content);
        Ok(key)
    }

    /// Get varchar field for an account
    pub async fn get_varchar(&self, account_id: &str, field_name: &str) -> Result<Option<String>> {
        let key = format!("{}:{}", account_id, field_name);

        match self.records_tree.get(&key)? {
            Some(data) => {
                let record: VarCharRecord = serde_json::from_slice(&data)?;
                Ok(Some(record.content))
            }
            None => Ok(None),
        }
    }

    /// Get all varchar fields for an account
    pub async fn get_account_varchars(&self, account_id: &str) -> Result<HashMap<String, String>> {
        let account_key = format!("account:{}", account_id);
        let mut result = HashMap::new();

        if let Some(fields_data) = self.accounts_tree.get(&account_key)? {
            let fields: Vec<String> = serde_json::from_slice(&fields_data)?;

            for field_name in fields {
                if let Some(content) = self.get_varchar(account_id, &field_name).await? {
                    result.insert(field_name, content);
                }
            }
        }

        Ok(result)
    }

    /// Update varchar field (creates if doesn't exist)
    pub async fn update_varchar(
        &self,
        account_id: &str,
        field_name: &str,
        new_content: &str,
    ) -> Result<()> {
        let key = format!("{}:{}", account_id, field_name);

        if let Some(existing_data) = self.records_tree.get(&key)? {
            let mut record: VarCharRecord = serde_json::from_slice(&existing_data)?;
            record.content = new_content.to_string();
            record.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            self.records_tree
                .insert(&key, serde_json::to_vec(&record)?)?;
            self.db.flush()?;
        } else {
            // Create new record
            self.store_varchar(account_id, field_name, new_content, "text", HashMap::new())
                .await?;
        }

        Ok(())
    }

    /// Delete varchar field
    pub async fn delete_varchar(&self, account_id: &str, field_name: &str) -> Result<bool> {
        let key = format!("{}:{}", account_id, field_name);
        let removed = self.records_tree.remove(&key)?.is_some();

        if removed {
            // Update account index
            let account_key = format!("account:{}", account_id);
            if let Some(fields_data) = self.accounts_tree.get(&account_key)? {
                let mut fields: Vec<String> = serde_json::from_slice(&fields_data)?;
                fields.retain(|f| f != field_name);

                if fields.is_empty() {
                    self.accounts_tree.remove(&account_key)?;
                } else {
                    self.accounts_tree
                        .insert(&account_key, serde_json::to_vec(&fields)?)?;
                }
            }

            self.db.flush()?;
        }

        Ok(removed)
    }

    /// Search content by hash (for deduplication)
    pub async fn find_by_content_hash(&self, content: &str) -> Result<Vec<String>> {
        let content_hash = Self::hash_content(content);
        let hash_key = format!("hash:{}", content_hash);

        match self.content_hash_tree.get(&hash_key)? {
            Some(data) => {
                let account_keys: Vec<String> = serde_json::from_slice(&data)?;
                Ok(account_keys)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<HashMap<String, u64>> {
        let mut stats = HashMap::new();

        stats.insert("total_records".to_string(), self.records_tree.len() as u64);
        stats.insert(
            "total_accounts".to_string(),
            self.accounts_tree.len() as u64,
        );
        stats.insert(
            "unique_content_hashes".to_string(),
            self.content_hash_tree.len() as u64,
        );
        stats.insert("db_size_bytes".to_string(), self.db.size_on_disk()? as u64);

        Ok(stats)
    }

    /// Compact database
    pub async fn compact(&self) -> Result<()> {
        info!("🗜️ Compacting SLED database...");
        self.db.flush()?;
        Ok(())
    }

    /// Hash content for deduplication
    fn hash_content(content: &str) -> i64 {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();

        // Take first 8 bytes and convert to i64
        let bytes: [u8; 8] = result[0..8].try_into().unwrap();
        i64::from_be_bytes(bytes)
    }
}

/// 🦖 Enhanced ZIK_ZAK Engine with SLED VARCHAR support
pub struct ZikZakSledEngine {
    pub accounting: crate::zik_zak::ZikZakEngine,
    pub varchar_store: SledVarCharStore,
}

impl ZikZakSledEngine {
    /// Initialize ZIK_ZAK with both TigerBeetle and SLED
    pub async fn new<P: AsRef<Path>>(sled_db_path: P) -> Result<Self> {
        let accounting = crate::zik_zak::ZikZakEngine::new().await?;
        let varchar_store = SledVarCharStore::new(sled_db_path)?;

        Ok(Self {
            accounting,
            varchar_store,
        })
    }

    /// Create product with both numeric and varchar data
    pub async fn create_product(
        &mut self,
        product_id: &str,
        name: &str,
        description: &str,
        price_cents: i64,
        category: &str,
    ) -> Result<String> {
        // 1. Create product existence (numeric)
        let existence_account = format!("product:{}:existence", product_id);
        self.accounting
            .transfer("system:genesis", &existence_account, 1, HashMap::new())
            .await?;

        // 2. Set price (numeric)
        let price_account = format!("product:{}:price", product_id);
        self.accounting
            .transfer(
                "system:genesis",
                &price_account,
                price_cents,
                HashMap::new(),
            )
            .await?;

        // 3. Store text fields (varchar)
        let base_account = format!("product:{}", product_id);

        self.varchar_store
            .store_varchar(
                &base_account,
                "name",
                name,
                "text",
                HashMap::from([("field_type".to_string(), "product_name".to_string())]),
            )
            .await?;

        self.varchar_store
            .store_varchar(
                &base_account,
                "description",
                description,
                "text",
                HashMap::from([("field_type".to_string(), "product_description".to_string())]),
            )
            .await?;

        self.varchar_store
            .store_varchar(
                &base_account,
                "category",
                category,
                "text",
                HashMap::from([("field_type".to_string(), "product_category".to_string())]),
            )
            .await?;

        info!(
            "🛍️ Created product: {} ({}) - ${:.2}",
            product_id,
            name,
            price_cents as f64 / 100.0
        );
        Ok(product_id.to_string())
    }

    /// Get complete product data
    pub async fn get_product(&self, product_id: &str) -> Result<Option<serde_json::Value>> {
        // Check if product exists
        let existence_account = format!("product:{}:existence", product_id);
        let exists = self.accounting.get_balance(&existence_account).await? > 0;

        if !exists {
            return Ok(None);
        }

        // Get price
        let price_account = format!("product:{}:price", product_id);
        let price = self.accounting.get_balance(&price_account).await?;

        // Get varchar fields
        let base_account = format!("product:{}", product_id);
        let varchar_fields = self
            .varchar_store
            .get_account_varchars(&base_account)
            .await?;

        let product_data = serde_json::json!({
            "id": product_id,
            "price_cents": price,
            "price_dollars": price as f64 / 100.0,
            "name": varchar_fields.get("name").unwrap_or(&"Unknown".to_string()),
            "description": varchar_fields.get("description").unwrap_or(&"No description".to_string()),
            "category": varchar_fields.get("category").unwrap_or(&"Uncategorized".to_string()),
            "exists": true,
        });

        Ok(Some(product_data))
    }

    /// Update product text field
    pub async fn update_product_field(
        &mut self,
        product_id: &str,
        field_name: &str,
        new_value: &str,
    ) -> Result<()> {
        let base_account = format!("product:{}", product_id);
        self.varchar_store
            .update_varchar(&base_account, field_name, new_value)
            .await?;
        Ok(())
    }

    /// Get system statistics
    pub async fn get_system_stats(&self) -> Result<serde_json::Value> {
        let account_count = self.accounting.get_account_count().await?;
        let transfer_count = self.accounting.get_transfer_count().await?;
        let varchar_stats = self.varchar_store.get_stats().await?;

        Ok(serde_json::json!({
            "tigerbeetle": {
                "accounts": account_count,
                "transfers": transfer_count,
                "connected": self.accounting.is_connected(),
            },
            "sled_varchar": varchar_stats,
            "total_storage": "hybrid_tigerbeetle_sled"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sled_varchar_storage() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_sled.db");

        let store = SledVarCharStore::new(&db_path)?;

        // Store varchar data
        store
            .store_varchar(
                "user:123",
                "name",
                "John Doe",
                "text",
                HashMap::from([("type".to_string(), "user_name".to_string())]),
            )
            .await?;

        store
            .store_varchar(
                "user:123",
                "email",
                "john@example.com",
                "email",
                HashMap::new(),
            )
            .await?;

        // Retrieve data
        let name = store.get_varchar("user:123", "name").await?;
        assert_eq!(name, Some("John Doe".to_string()));

        let email = store.get_varchar("user:123", "email").await?;
        assert_eq!(email, Some("john@example.com".to_string()));

        // Get all fields for account
        let all_fields = store.get_account_varchars("user:123").await?;
        assert_eq!(all_fields.len(), 2);
        assert_eq!(all_fields.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(
            all_fields.get("email"),
            Some(&"john@example.com".to_string())
        );

        // Update field
        store.update_varchar("user:123", "name", "Jane Doe").await?;
        let updated_name = store.get_varchar("user:123", "name").await?;
        assert_eq!(updated_name, Some("Jane Doe".to_string()));

        // Delete field
        let deleted = store.delete_varchar("user:123", "email").await?;
        assert!(deleted);

        let email_after_delete = store.get_varchar("user:123", "email").await?;
        assert_eq!(email_after_delete, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_complete_zik_zak_sled_engine() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test_complete.db");

        let mut engine = ZikZakSledEngine::new(&db_path).await?;

        // Create product with both numeric and varchar data
        let product_id = engine
            .create_product(
                "12345",
                "ZIK_ZAK T-Shirt",
                "Revolutionary accounting engine merchandise",
                2999, // $29.99
                "Apparel",
            )
            .await?;

        // Verify product exists and get complete data
        let product = engine.get_product(&product_id).await?;
        assert!(product.is_some());

        let product_data = product.unwrap();
        assert_eq!(product_data["id"], "12345");
        assert_eq!(product_data["price_cents"], 2999);
        assert_eq!(product_data["name"], "ZIK_ZAK T-Shirt");
        assert_eq!(product_data["category"], "Apparel");

        // Update product description
        engine
            .update_product_field(
                "12345",
                "description",
                "Updated: The best accounting engine merch!",
            )
            .await?;

        // Verify update
        let updated_product = engine.get_product("12345").await?.unwrap();
        assert_eq!(
            updated_product["description"],
            "Updated: The best accounting engine merch!"
        );

        // Get system stats
        let stats = engine.get_system_stats().await?;
        assert!(stats["tigerbeetle"]["accounts"].as_u64().unwrap() > 0);
        assert!(stats["sled_varchar"]["total_records"].as_u64().unwrap() > 0);

        Ok(())
    }
}
