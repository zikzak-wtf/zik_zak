//! Integration test using real TigerBeetle
//! 
//! This test demonstrates the ZIK_ZAK revolution with actual TigerBeetle operations.
//! 
//! ## Requirements
//! 
//! Before running this test, ensure TigerBeetle is running:
//! ```bash
//! # Terminal 1: Start TigerBeetle server
//! tigerbeetle start --cluster-id=0 --addresses=3000 0_0.tigerbeetle
//! 
//! # Terminal 2: Run the test
//! cargo test tigerbeetle_integration_test --release
//! ```

use anyhow::Result;
use std::collections::HashMap;
use zik_zak::{Genesis, ZikZak, zik, zak};

#[tokio::test]
async fn test_simple_tigerbeetle_operations() -> Result<()> {
    println!("🦖 Starting ZIK_ZAK TigerBeetle Integration Test");
    println!("====================================================");

    // Create a temporary database for this test
    let temp_dir = tempfile::TempDir::new()?;
    let sled_path = temp_dir.path().join("test_genesis.db");

    // Initialize GENESIS - The Divine Creator
    let mut genesis = Genesis::empty(sled_path).await?;

    println!("✨ GENESIS initialized successfully");
    println!("🔌 TigerBeetle connected: {}", genesis.is_divine());

    // Test 1: Basic account creation through transfers
    println!("\n🧪 Test 1: Creating entities through divine transfers");
    
    // Create a product by transferring existence from genesis
    let product_transfer = genesis.accounting.transfer(
        "system:genesis",
        "product:123:existence", 
        1,
        HashMap::new()
    ).await?;
    
    println!("📦 Created product:123 - Transfer ID: {}", product_transfer);

    // Set the product price
    let price_transfer = genesis.accounting.transfer(
        "system:genesis",
        "product:123:price",
        2999, // $29.99 in cents
        HashMap::new()
    ).await?;
    
    println!("💰 Set product:123 price to $29.99 - Transfer ID: {}", price_transfer);

    // Test 2: Check balances (the only read operation we need)
    println!("\n🧪 Test 2: Reading entity state through balances");

    let product_exists = genesis.accounting.get_balance("product:123:existence").await?;
    let product_price = genesis.accounting.get_balance("product:123:price").await?;

    println!("🔍 product:123:existence balance: {} (1 = exists)", product_exists);
    println!("🔍 product:123:price balance: {} cents", product_price);

    assert_eq!(product_exists, 1, "Product should exist");
    assert_eq!(product_price, 2999, "Product price should be $29.99");

    // Test 3: Create a user with balance
    println!("\n🧪 Test 3: Creating user account with balance");

    let user_creation = genesis.accounting.transfer(
        "system:genesis",
        "user:456:existence",
        1,
        HashMap::new()
    ).await?;

    let user_funding = genesis.accounting.transfer(
        "system:genesis", 
        "user:456:balance",
        10000, // $100.00 in cents
        HashMap::new()
    ).await?;

    println!("👤 Created user:456 - Transfer ID: {}", user_creation);
    println!("💳 Funded user:456 with $100.00 - Transfer ID: {}", user_funding);

    let user_balance = genesis.accounting.get_balance("user:456:balance").await?;
    println!("🔍 user:456:balance: {} cents", user_balance);
    assert_eq!(user_balance, 10000, "User should have $100.00");

    // Test 4: Execute a purchase (user buys product)
    println!("\n🧪 Test 4: User purchasing product (pure accounting)");

    // Transfer money from user to merchant
    let purchase_transfer = genesis.accounting.transfer(
        "user:456:balance",
        "merchant:789:revenue",
        2999, // Product price
        {
            let mut metadata = HashMap::new();
            metadata.insert("product_id".to_string(), "123".to_string());
            metadata.insert("transaction_type".to_string(), "purchase".to_string());
            metadata
        }
    ).await?;

    println!("🛒 Purchase completed - Transfer ID: {}", purchase_transfer);

    // Check final balances
    let final_user_balance = genesis.accounting.get_balance("user:456:balance").await?;
    let merchant_revenue = genesis.accounting.get_balance("merchant:789:revenue").await?;

    println!("🔍 Final user:456:balance: {} cents", final_user_balance);
    println!("🔍 merchant:789:revenue: {} cents", merchant_revenue);

    assert_eq!(final_user_balance, 7001, "User should have $70.01 left");
    assert_eq!(merchant_revenue, 2999, "Merchant should have $29.99 revenue");

    // Test 5: Get ledger state - the entire "database"
    println!("\n🧪 Test 5: Divine ledger state (the entire backend state)");

    let ledger_state = genesis.divine_ledger().await?;
    println!("📊 Current ledger state:\n{}", serde_json::to_string_pretty(&ledger_state)?);

    // Test 6: Divine statistics
    println!("\n🧪 Test 6: GENESIS divine statistics");

    let stats = genesis.divine_stats().await?;
    println!("📈 Divine stats:\n{}", serde_json::to_string_pretty(&stats)?);

    println!("\n🎉 ALL TESTS PASSED! The ZIK_ZAK revolution is real!");
    println!("====================================================");
    println!("🦖 Backend development is officially DEAD. 💀");
    println!("✨ Pure accounting has replaced your entire tech stack.");
    println!("🔥 Welcome to the future of software development!");

    Ok(())
}

#[tokio::test]
async fn test_complex_business_logic_with_pure_accounting() -> Result<()> {
    println!("🧠 Testing Complex Business Logic with Pure Accounting");
    println!("====================================================");

    let temp_dir = tempfile::TempDir::new()?;
    let sled_path = temp_dir.path().join("complex_test.db");

    let mut genesis = Genesis::empty(sled_path).await?;

    // Scenario: Multi-user e-commerce with inventory management
    println!("🏪 Creating e-commerce scenario with inventory management");

    // Create inventory by transferring stock from genesis
    for product_id in 1..=3 {
        genesis.accounting.transfer(
            "system:genesis",
            &format!("inventory:product_{}:stock", product_id),
            100, // 100 units in stock
            HashMap::new()
        ).await?;

        genesis.accounting.transfer(
            "system:genesis",
            &format!("inventory:product_{}:price", product_id),
            (1000 + product_id * 500) as i64, // Varying prices
            HashMap::new()
        ).await?;
    }

    // Create multiple users with different balances
    for user_id in 1..=3 {
        genesis.accounting.transfer(
            "system:genesis",
            &format!("user:{}:balance", user_id),
            (5000 + user_id * 2000) as i64, // $50, $70, $90
            HashMap::new()
        ).await?;
    }

    println!("📦 Inventory and users created");

    // Execute multiple purchase transactions
    let purchase_scenarios = vec![
        (1, 1, 2), // User 1 buys 2 units of product 1
        (2, 2, 1), // User 2 buys 1 unit of product 2  
        (3, 3, 3), // User 3 buys 3 units of product 3
        (1, 2, 1), // User 1 buys 1 unit of product 2
    ];

    for (user_id, product_id, quantity) in purchase_scenarios {
        let price = genesis.accounting.get_balance(&format!("inventory:product_{}:price", product_id)).await?;
        let total_cost = price * quantity;

        println!("🛒 User {} buying {} units of product {} for ${:.2}", 
                user_id, quantity, product_id, total_cost as f64 / 100.0);

        // Deduct inventory
        genesis.accounting.transfer(
            &format!("inventory:product_{}:stock", product_id),
            &format!("inventory:product_{}:sold", product_id),
            quantity,
            HashMap::new()
        ).await?;

        // Process payment
        genesis.accounting.transfer(
            &format!("user:{}:balance", user_id),
            &format!("merchant:revenue:product_{}", product_id),
            total_cost,
            HashMap::new()
        ).await?;

        // Track user purchases
        genesis.accounting.transfer(
            "system:genesis",
            &format!("user:{}:purchases:product_{}", user_id, product_id),
            quantity,
            HashMap::new()
        ).await?;
    }

    // Print final state - this IS the database
    println!("\n📊 Final Business State (Pure Accounting):");

    // Check inventory levels
    for product_id in 1..=3 {
        let remaining_stock = genesis.accounting.get_balance(&format!("inventory:product_{}:stock", product_id)).await?;
        let sold_units = genesis.accounting.get_balance(&format!("inventory:product_{}:sold", product_id)).await?;
        let price = genesis.accounting.get_balance(&format!("inventory:product_{}:price", product_id)).await?;
        
        println!("📦 Product {}: {} units left, {} sold, price ${:.2}", 
               product_id, remaining_stock, sold_units, price as f64 / 100.0);
    }

    // Check user balances and purchase history
    for user_id in 1..=3 {
        let balance = genesis.accounting.get_balance(&format!("user:{}:balance", user_id)).await?;
        println!("👤 User {}: ${:.2} remaining", user_id, balance as f64 / 100.0);
        
        for product_id in 1..=3 {
            let purchases = genesis.accounting.get_balance(&format!("user:{}:purchases:product_{}", user_id, product_id)).await?;
            if purchases > 0 {
                println!("  📝 Purchased {} units of product {}", purchases, product_id);
            }
        }
    }

    // Check merchant revenue by product
    println!("\n💰 Merchant Revenue Analysis:");
    for product_id in 1..=3 {
        let revenue = genesis.accounting.get_balance(&format!("merchant:revenue:product_{}", product_id)).await?;
        println!("💵 Product {} revenue: ${:.2}", product_id, revenue as f64 / 100.0);
    }

    println!("\n🎯 COMPLEX BUSINESS LOGIC COMPLETED WITH PURE ACCOUNTING!");
    println!("No SQL. No ORM. No schemas. No migrations. No backend complexity.");
    println!("Just pure mathematical accounting that scales infinitely! 🚀");

    Ok(())
}