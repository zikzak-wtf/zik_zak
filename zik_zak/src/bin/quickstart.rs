//! # 🦖 ZIK_ZAK + SLED Quick Start Guide
//!
//! The fastest way to replace your entire backend with pure accounting + varchar storage.

use anyhow::Result;
use std::collections::HashMap;
use zik_zak::ZikZakSledEngine;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦖 ZIK_ZAK + SLED Quick Start");
    println!("==============================");

    // Initialize the complete backend replacement
    let mut engine = ZikZakSledEngine::new("./quickstart.db").await?;
    engine.accounting.ensure_system_accounts().await?;

    println!("\n✅ Backend initialized (TigerBeetle + SLED)");

    // ============================================================================
    // SECTION 1: Basic Product Management
    // ============================================================================
    println!("\n📦 1. Product Management");
    println!("------------------------");

    // Create products with both numeric and text data
    engine.create_product(
        "laptop-001",
        "Gaming Laptop",
        "High-performance laptop for gaming and development",
        149999, // $1,499.99
        "Electronics",
    ).await?;

    engine.create_product(
        "book-001", 
        "The ZIK_ZAK Manifesto",
        "The book that ended backend development forever",
        1999, // $19.99
        "Books",
    ).await?;

    // Query complete product data
    let laptop = engine.get_product("laptop-001").await?.unwrap();
    println!("Created: {} - ${}", laptop["name"], laptop["price_dollars"]);
    
    let book = engine.get_product("book-001").await?.unwrap();
    println!("Created: {} - ${}", book["name"], book["price_dollars"]);

    // ============================================================================
    // SECTION 2: User Management with Profiles
    // ============================================================================
    println!("\n👤 2. User Management");
    println!("---------------------");

    let users = [
        ("alice", "Alice Johnson", "alice@example.com", "Full-Stack Developer"),
        ("bob", "Bob Wilson", "bob@example.com", "DevOps Engineer"),
        ("charlie", "Charlie Brown", "charlie@example.com", "Product Manager"),
    ];

    for (user_id, name, email, role) in &users {
        let base_account = format!("user:{}", user_id);
        
        // Numeric data: Create user and give initial balance
        engine.accounting.transfer("system:genesis", &format!("{}:existence", base_account), 1, HashMap::new()).await?;
        engine.accounting.transfer("system:genesis", &format!("{}:balance", base_account), 50000, HashMap::new()).await?; // $500
        
        // Text data: Store profile information
        engine.varchar_store.store_varchar(&base_account, "name", name, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(&base_account, "email", email, "email", HashMap::new()).await?;
        engine.varchar_store.store_varchar(&base_account, "role", role, "text", HashMap::new()).await?;
        
        println!("Created user: {} ({}) - $500.00", name, email);
    }

    // ============================================================================
    // SECTION 3: Order Processing
    // ============================================================================
    println!("\n🛒 3. Order Processing");
    println!("----------------------");

    // Alice orders the laptop
    let order_id = "order:001";
    let laptop_price = engine.accounting.get_balance("product:laptop-001:price").await?;
    
    // Process payment
    engine.accounting.transfer(
        "user:alice:balance",
        "store:revenue", 
        laptop_price,
        HashMap::from([
            ("order_id".to_string(), order_id.to_string()),
            ("product_id".to_string(), "laptop-001".to_string()),
        ])
    ).await?;

    // Create order with status and details
    engine.accounting.transfer("system:genesis", &format!("{}:status", order_id), 1, HashMap::new()).await?; // 1 = pending
    engine.varchar_store.store_varchar(order_id, "user_id", "alice", "text", HashMap::new()).await?;
    engine.varchar_store.store_varchar(order_id, "product_id", "laptop-001", "text", HashMap::new()).await?;
    engine.varchar_store.store_varchar(order_id, "shipping_address", "123 Main St, San Francisco, CA", "text", HashMap::new()).await?;
    engine.varchar_store.store_varchar(order_id, "notes", "Please handle with care - expensive item!", "text", HashMap::new()).await?;

    println!("Order created: Alice purchased Gaming Laptop for ${:.2}", laptop_price as f64 / 100.0);

    // ============================================================================
    // SECTION 4: Reviews and Ratings
    // ============================================================================
    println!("\n⭐ 4. Reviews & Ratings");
    println!("-----------------------");

    // Alice reviews the laptop
    let review_id = "review:001";
    
    // Numeric: Rating (1-5 stars)
    engine.accounting.transfer("system:genesis", &format!("{}:rating", review_id), 5, HashMap::new()).await?;
    
    // Text: Review content
    engine.varchar_store.store_varchar(review_id, "user_id", "alice", "text", HashMap::new()).await?;
    engine.varchar_store.store_varchar(review_id, "product_id", "laptop-001", "text", HashMap::new()).await?;
    engine.varchar_store.store_varchar(review_id, "title", "Absolutely Amazing!", "text", HashMap::new()).await?;
    engine.varchar_store.store_varchar(review_id, "comment", "This laptop exceeded all my expectations. Perfect for development and gaming!", "text", HashMap::new()).await?;

    let rating = engine.accounting.get_balance(&format!("{}:rating", review_id)).await?;
    let review_comment = engine.varchar_store.get_varchar(review_id, "comment").await?.unwrap();
    println!("Review added: {} stars - \"{}\"", rating, review_comment);

    // ============================================================================
    // SECTION 5: Inventory Management
    // ============================================================================
    println!("\n📊 5. Inventory Management");
    println!("--------------------------");

    // Set initial inventory
    engine.accounting.transfer("system:genesis", "product:laptop-001:inventory", 25, HashMap::new()).await?;
    engine.accounting.transfer("system:genesis", "product:book-001:inventory", 100, HashMap::new()).await?;

    // Reduce inventory after sale
    engine.accounting.transfer("product:laptop-001:inventory", "system:sold", 1, HashMap::new()).await?;

    let laptop_inventory = engine.accounting.get_balance("product:laptop-001:inventory").await?;
    let book_inventory = engine.accounting.get_balance("product:book-001:inventory").await?;
    
    println!("Inventory: Gaming Laptop ({} units), ZIK_ZAK Book ({} units)", laptop_inventory, book_inventory);

    // ============================================================================
    // SECTION 6: User Account Summary
    // ============================================================================
    println!("\n👤 6. User Account Summary");
    println!("--------------------------");

    let alice_balance = engine.accounting.get_balance("user:alice:balance").await?;
    let alice_profile = engine.varchar_store.get_account_varchars("user:alice").await?;
    
    println!("Alice's Account:");
    println!("  Name: {}", alice_profile.get("name").unwrap());
    println!("  Email: {}", alice_profile.get("email").unwrap());
    println!("  Role: {}", alice_profile.get("role").unwrap());
    println!("  Balance: ${:.2}", alice_balance as f64 / 100.0);

    // ============================================================================
    // SECTION 7: System Analytics
    // ============================================================================
    println!("\n📈 7. System Analytics");
    println!("----------------------");

    let store_revenue = engine.accounting.get_balance("store:revenue").await?;
    let total_orders = 1; // We created 1 order
    let total_reviews = 1; // We created 1 review
    
    println!("Business Metrics:");
    println!("  Total Revenue: ${:.2}", store_revenue as f64 / 100.0);
    println!("  Total Orders: {}", total_orders);
    println!("  Total Reviews: {}", total_reviews);
    println!("  Average Order Value: ${:.2}", store_revenue as f64 / 100.0 / total_orders as f64);

    // ============================================================================
    // SECTION 8: System Statistics
    // ============================================================================
    println!("\n🔧 8. System Statistics");
    println!("-----------------------");

    let stats = engine.get_system_stats().await?;
    println!("System Health:");
    println!("  TigerBeetle Accounts: {}", stats["tigerbeetle"]["accounts"]);
    println!("  TigerBeetle Transfers: {}", stats["tigerbeetle"]["transfers"]);
    println!("  SLED Records: {}", stats["sled_varchar"]["total_records"]);
    println!("  SLED Database Size: {} bytes", stats["sled_varchar"]["db_size_bytes"]);

    // ============================================================================
    // CONCLUSION
    // ============================================================================
    println!("\n🎉 SUCCESS: Complete Backend Replacement");
    println!("========================================");
    println!("✅ Product catalog with prices and descriptions");
    println!("✅ User management with profiles and balances");
    println!("✅ Order processing with payment tracking");
    println!("✅ Review system with ratings and comments");
    println!("✅ Inventory management with stock tracking");
    println!("✅ Revenue analytics and business metrics");
    println!("✅ All without traditional databases, APIs, or schemas!");
    
    println!("\n🦖 ZIK_ZAK + SLED = The End of Backend Development");
    println!("   TigerBeetle handles all numeric data");
    println!("   SLED handles all varchar/text data");
    println!("   Pure accounting + key-value storage");
    println!("   Zero complexity, infinite scale");

    Ok(())
}