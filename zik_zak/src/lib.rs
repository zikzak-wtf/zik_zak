//! # 🦖 ZIK_ZAK - The End of Backend Development Forever
//!
//! ## The Revolution
//!
//! ZIK_ZAK eliminates traditional backend development by replacing:
//! - ❌ Databases with accounting
//! - ❌ Schemas with flexibility
//! - ❌ APIs with transfers
//! - ❌ Frameworks with math
//!
//! ## Core Concepts
//!
//! Everything is an **account** with a **balance**:
//! ```ignore
//! product:123:price = 2999      // Product costs $29.99
//! user:456:balance = 10000      // User has $100.00
//! order:789:status = 1          // Order is active
//! ```
//!
//! Every operation is a **transfer**:
//! ```ignore
//! transfer(genesis, product:123:existence, 1)     // Create product
//! transfer(genesis, product:123:price, 2999)      // Set price
//! transfer(user:456, merchant:789, 2999)          // Purchase
//! transfer(entity, system:deleted, balance)       // Delete
//! ```
//!
//! ## The Entire Backend
//!
//! ```rust
//! use zik_zak::{AccountingEngine, Result};
//!
//! # async fn example() -> Result<()> {
//! let mut engine = AccountingEngine::new().await?;
//!
//! // Get balance - the only read operation you need
//! let balance = engine.get_balance("product:123:price").await?;
//!
//! // Transfer - the only write operation you need
//! let transfer_id = engine.transfer(
//!     "user:456",
//!     "merchant:789",
//!     2999,
//!     Default::default()
//! ).await?;
//!
//! // That's it. That's the entire backend.
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - 🚀 **Zero schemas** - Add any field instantly
//! - ⚡ **Real-time** - Balance changes are events
//! - 🔒 **ACID guarantees** - Double-entry accounting
//! - 📊 **Perfect audit** - Every operation logged
//! - 🦖 **Infinite scale** - TigerBeetle performance
//!
//! ## Quick Start
//!
//! ```bash
//! cargo run --bin zik_zak
//! curl localhost:3002/health
//! ```
//!
//! Welcome to the revolution. 🔥

pub mod accounting;
pub mod recipes;
pub mod tigerbeetle_client;

pub use accounting::{Transfer, ZikZakEngine};
pub use recipes::{Recipe, RecipeEngine};
pub use tigerbeetle_client::TigerBeetleClient;

/// Result type used throughout ZIK_ZAK
pub type Result<T> = anyhow::Result<T>;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The revolution manifesto
pub const MANIFESTO: &str = r#"
🦖 ZIK_ZAK MANIFESTO

The best code is no code.
The best database is no database.
The best API is no API.
The best schema is no schema.

Everything is accounting.

Backend development is dead.
We killed it.
You're welcome.
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifesto_contains_truth() {
        assert!(MANIFESTO.contains("Everything is accounting"));
        assert!(MANIFESTO.contains("Backend development is dead"));
    }

    #[test]
    fn test_version_exists() {
        assert!(!VERSION.is_empty());
    }
}
