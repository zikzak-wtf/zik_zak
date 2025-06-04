//! # ⚡ ZIK_ZAK Genesis Example - Divine Spark Ignition
//!
//! Demonstrates the revolutionary GENESIS API that replaces entire backends
//! with pure accounting and divine sparks.

use anyhow::Result;
use tempfile::TempDir;
use zik_zak::{zak, zik, Genesis, ZikZak};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize divine logging
    tracing_subscriber::init();

    println!("🌟 Initializing GENESIS - The Divine Creator");

    // Create temporary directory for this divine demonstration
    let temp_dir = TempDir::new()?;
    let genesis_db_path = temp_dir.path().join("genesis.db");

    // Initialize GENESIS with divine sparks
    let mut genesis = Genesis::new("divine_sparks.json", genesis_db_path).await?;

    println!(
        "✨ GENESIS is ready! Divine status: {}",
        genesis.is_divine()
    );

    // ⚡ DIVINE CREATION - Birth a user into existence
    println!("\n⚡ Igniting 'create_user' spark...");
    let user_result = genesis
        .ignite_spark(
            "create_user",
            ZikZak {
                zik: zik! {}, // Nothing flows out from user during creation
                zak: zak! {    // User receives existence and identity
                    user_id: 123,
                    username: "alice",
                    email: "alice@divine.com"
                },
            },
        )
        .await?;

    println!("👤 User created: {:?}", user_result);

    // ⚡ DIVINE MANIFESTATION - Create a product
    println!("\n⚡ Igniting 'create_product' spark...");
    let product_result = genesis
        .ignite_spark(
            "create_product",
            ZikZak {
                zik: zik! {}, // Nothing flows out
                zak: zak! {    // Product receives existence and properties
                    product_id: 456,
                    name: "Divine Widget",
                    price: 2999,
                    description: "A widget blessed by GENESIS"
                },
            },
        )
        .await?;

    println!("📦 Product created: {:?}", product_result);

    // ⚡ DIVINE TRANSACTION - Create an order (value exchange)
    println!("\n⚡ Igniting 'create_order' spark...");
    let order_result = genesis
        .ignite_spark(
            "create_order",
            ZikZak {
                zik: zik! {    // User gives payment
                    user_id: 123,
                    price: 2999
                },
                zak: zak! {    // User receives order
                    product_id: 456,
                    order_id: 789,
                    quantity: 1
                },
            },
        )
        .await?;

    println!("📋 Order created: {:?}", order_result);

    // 🔍 DIVINE QUERY - Retrieve user information
    println!("\n🔍 Igniting 'get_user' spark...");
    let user_info = genesis
        .ignite_spark(
            "get_user",
            ZikZak {
                zik: zik! { user_id: 123 }, // Query for this user
                zak: zak! {},               // Expect user data back
            },
        )
        .await?;

    println!("👤 User info: {:?}", user_info);

    // 📊 DIVINE STATISTICS - See GENESIS's creative power
    println!("\n📊 Divine Statistics:");
    let stats = genesis.divine_stats().await?;
    println!("{}", serde_json::to_string_pretty(&stats)?);

    // 🔮 DIVINE LEDGER - Current state of reality
    println!("\n🔮 Divine Ledger (Current Reality):");
    let ledger = genesis.divine_ledger().await?;
    println!("{}", serde_json::to_string_pretty(&ledger)?);

    println!("\n🦖 REVOLUTION COMPLETE!");
    println!("Backend development is dead. GENESIS has replaced it with divine sparks.");
    println!("No controllers. No services. No repositories. Just pure accounting magic.");

    Ok(())
}
