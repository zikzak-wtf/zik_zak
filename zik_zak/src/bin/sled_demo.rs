//! # 🦖 ZIK_ZAK SLED Integration Example
//!
//! Demonstrates the complete ZIK_ZAK + SLED solution for varchar fields

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use zik_zak::ZikZakSledEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the hybrid engine
    println!("🦖 Initializing ZIK_ZAK with SLED varchar storage...");
    let mut engine = ZikZakSledEngine::new("./zik_zak_sled.db").await?;

    // Ensure system accounts exist
    engine.accounting.ensure_system_accounts().await?;

    println!("\n🛍️ Creating products with both numeric and varchar data...");

    // Create complete products
    let products = vec![
        ("001", "ZIK_ZAK Hoodie", "Revolutionary accounting engine hoodie", 4999, "Apparel"),
        ("002", "TigerBeetle Mug", "Coffee mug for financial engineers", 1999, "Accessories"),
        ("003", "SLED Database Book", "Learn embedded databases", 3999, "Books"),
        ("004", "Accounting Revolution", "The manifesto that changed everything", 2499, "Books"),
    ];

    for (id, name, desc, price, category) in products {
        engine.create_product(id, name, desc, price, category).await?;
    }

    println!("\n📊 Product catalog:");
    for i in 1..=4 {
        let product_id = format!("{:03}", i);
        if let Some(product) = engine.get_product(&product_id).await? {
            println!("  {} - {} (${:.2})", 
                product["name"], 
                product["category"], 
                product["price_dollars"].as_f64().unwrap()
            );
        }
    }

    println!("\n👤 Creating users with profiles...");

    // Create users with varchar profiles
    let users = vec![
        ("alice", "Alice Johnson", "alice@example.com", "Software Engineer", "San Francisco"),
        ("bob", "Bob Smith", "bob@example.com", "Product Manager", "New York"),
        ("charlie", "Charlie Brown", "charlie@example.com", "Designer", "Los Angeles"),
    ];

    for (user_id, name, email, role, city) in users {
        let base_account = format!("user:{}", user_id);
        
        // Create user existence
        engine.accounting.transfer(
            "system:genesis",
            &format!("{}:existence", base_account),
            1,
            HashMap::new(),
        ).await?;

        // Give user initial balance
        engine.accounting.transfer(
            "system:genesis",
            &format!("{}:balance", base_account),
            10000, // $100.00
            HashMap::new(),
        ).await?;

        // Store user profile in SLED
        engine.varchar_store.store_varchar(&base_account, "name", name, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(&base_account, "email", email, "email", HashMap::new()).await?;
        engine.varchar_store.store_varchar(&base_account, "role", role, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(&base_account, "city", city, "text", HashMap::new()).await?;

        println!("  Created user: {} ({}) - $100.00 balance", name, email);
    }

    println!("\n🛒 Processing orders...");

    // Create orders with both accounting and varchar data
    let orders = vec![
        ("order:001", "alice", "001", 1, "Express shipping please!"),
        ("order:002", "bob", "002", 2, "Gift wrapping needed"),
        ("order:003", "charlie", "003", 1, "Standard shipping is fine"),
    ];

    for (order_id, user_id, product_id, quantity, notes) in orders {
        // Get product price
        let price_account = format!("product:{}:price", product_id);
        let unit_price = engine.accounting.get_balance(&price_account).await?;
        let total_price = unit_price * quantity;

        // Process payment (transfer from user to merchant)
        let user_balance_account = format!("user:{}:balance", user_id);
        engine.accounting.transfer(
            &user_balance_account,
            "merchant:zikzak:revenue",
            total_price,
            HashMap::from([
                ("order_id".to_string(), order_id.to_string()),
                ("product_id".to_string(), product_id.to_string()),
            ]),
        ).await?;

        // Create order status
        engine.accounting.transfer(
            "system:genesis",
            &format!("{}:status", order_id),
            1, // 1 = pending, 2 = shipped, 3 = delivered
            HashMap::new(),
        ).await?;

        // Store order details in SLED
        engine.varchar_store.store_varchar(order_id, "user_id", user_id, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(order_id, "product_id", product_id, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(order_id, "notes", notes, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(order_id, "quantity", &quantity.to_string(), "number", HashMap::new()).await?;

        if let Some(product) = engine.get_product(product_id).await? {
            println!("  {} ordered {} x {} (${:.2})", 
                user_id, 
                quantity,
                product["name"], 
                total_price as f64 / 100.0
            );
        }
    }

    println!("\n💬 Adding product reviews...");

    // Create reviews with text content
    let reviews = vec![
        ("review:001", "alice", "001", 5, "Amazing hoodie! The ZIK_ZAK logo is perfect."),
        ("review:002", "bob", "002", 4, "Great mug, perfect for morning coffee."),
        ("review:003", "charlie", "003", 5, "This book completely changed how I think about databases!"),
    ];

    for (review_id, user_id, product_id, rating, comment) in reviews {
        // Store rating as accounting balance
        engine.accounting.transfer(
            "system:genesis",
            &format!("{}:rating", review_id),
            rating,
            HashMap::new(),
        ).await?;

        // Store review text in SLED
        engine.varchar_store.store_varchar(review_id, "user_id", user_id, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(review_id, "product_id", product_id, "text", HashMap::new()).await?;
        engine.varchar_store.store_varchar(review_id, "comment", comment, "text", HashMap::new()).await?;

        println!("  {} rated product {} - {} stars: {}", user_id, product_id, rating, comment);
    }

    println!("\n📈 System Statistics:");
    let stats = engine.get_system_stats().await?;
    println!("{}", serde_json::to_string_pretty(&stats)?);

    println!("\n🔍 Querying complete data...");

    // Show Alice's profile and balance
    let alice_balance = engine.accounting.get_balance("user:alice:balance").await?;
    let alice_profile = engine.varchar_store.get_account_varchars("user:alice").await?;
    
    println!("\n👤 Alice's Profile:");
    println!("  Name: {}", alice_profile.get("name").unwrap_or(&"Unknown".to_string()));
    println!("  Email: {}", alice_profile.get("email").unwrap_or(&"Unknown".to_string()));
    println!("  Role: {}", alice_profile.get("role").unwrap_or(&"Unknown".to_string()));
    println!("  City: {}", alice_profile.get("city").unwrap_or(&"Unknown".to_string()));
    println!("  Balance: ${:.2}", alice_balance as f64 / 100.0);

    // Show order details
    println!("\n📦 Order Details:");
    let order_status = engine.accounting.get_balance("order:001:status").await?;
    let order_details = engine.varchar_store.get_account_varchars("order:001").await?;
    
    println!("  Order ID: order:001");
    println!("  User: {}", order_details.get("user_id").unwrap_or(&"Unknown".to_string()));
    println!("  Product: {}", order_details.get("product_id").unwrap_or(&"Unknown".to_string()));
    println!("  Quantity: {}", order_details.get("quantity").unwrap_or(&"0".to_string()));
    println!("  Notes: {}", order_details.get("notes").unwrap_or(&"None".to_string()));
    println!("  Status: {} (1=pending, 2=shipped, 3=delivered)", order_status);

    println!("\n🎉 ZIK_ZAK + SLED Integration Complete!");
    println!("✅ TigerBeetle handles all numeric accounting");
    println!("✅ SLED handles all varchar/text fields");
    println!("✅ No traditional database needed!");
    println!("✅ No schema migrations required!");
    println!("✅ Infinite scalability achieved!");

    Ok(())
}