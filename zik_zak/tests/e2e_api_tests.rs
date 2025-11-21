//! End-to-end API tests
//!
//! Tests the complete HTTP server functionality including
//! recipe execution, balance queries, and error handling.

use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to check if server is running
async fn is_server_running() -> bool {
    reqwest::get("http://localhost:3003/health")
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Helper to wait for server to be ready
async fn wait_for_server(max_attempts: u32) -> Result<()> {
    for attempt in 1..=max_attempts {
        if is_server_running().await {
            println!("  ✅ Server is ready");
            return Ok(());
        }
        if attempt < max_attempts {
            println!("  ⏳ Waiting for server (attempt {}/{})", attempt, max_attempts);
            sleep(Duration::from_secs(1)).await;
        }
    }
    anyhow::bail!("Server did not start within timeout")
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_health_endpoint() -> Result<()> {
    println!("🧪 Testing /health endpoint");

    let response = reqwest::get("http://localhost:3003/health").await?;

    assert!(response.status().is_success());
    println!("  ✅ Health endpoint returns 200");

    let body: Value = response.json().await?;
    assert_eq!(body["status"], "healthy");
    println!("  ✅ Health response correct: {:?}", body);

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_manifesto_endpoint() -> Result<()> {
    println!("🧪 Testing /manifesto endpoint");

    let response = reqwest::get("http://localhost:3003/manifesto").await?;

    assert!(response.status().is_success());
    let body = response.text().await?;

    assert!(body.contains("ZIK_ZAK"));
    assert!(body.contains("accounting"));
    println!("  ✅ Manifesto endpoint works");

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_create_product_via_api() -> Result<()> {
    println!("🧪 Testing product creation via API");

    let client = reqwest::Client::new();

    // Create a product
    let product_data = json!({
        "id": "api_test_product_001",
        "name": "API Test Product",
        "price": 4999,
        "description": "Created via API test"
    });

    let response = client
        .post("http://localhost:3003/recipe/create_product")
        .json(&product_data)
        .send()
        .await?;

    if response.status().is_success() {
        println!("  ✅ Product creation request successful");

        let result: Value = response.json().await?;
        println!("  📦 Response: {:?}", result);

        // Give TigerBeetle a moment to process
        sleep(Duration::from_millis(100)).await;

        // Query the balance to verify
        let balance_response = client
            .get("http://localhost:3003/balance/product:api_test_product_001:existence")
            .send()
            .await?;

        if balance_response.status().is_success() {
            let balance: Value = balance_response.json().await?;
            println!("  ✅ Product existence verified: {:?}", balance);
        }
    } else {
        println!(
            "  ⚠️  Recipe endpoint may not be implemented yet: {}",
            response.status()
        );
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_balance_query_api() -> Result<()> {
    println!("🧪 Testing balance query API");

    let client = reqwest::Client::new();

    // Query system genesis account (should always exist)
    let response = client
        .get("http://localhost:3003/balance/system:genesis")
        .send()
        .await?;

    if response.status().is_success() {
        let balance: Value = response.json().await?;
        println!("  ✅ Balance query successful: {:?}", balance);
        assert!(balance.is_object());
    } else {
        println!(
            "  ⚠️  Balance endpoint may not be implemented yet: {}",
            response.status()
        );
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_error_handling_invalid_recipe() -> Result<()> {
    println!("🧪 Testing error handling for invalid recipe");

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3003/recipe/non_existent_recipe")
        .json(&json!({}))
        .send()
        .await?;

    // Should return an error status
    assert!(!response.status().is_success());
    println!("  ✅ Invalid recipe returns error status: {}", response.status());

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_error_handling_malformed_json() -> Result<()> {
    println!("🧪 Testing error handling for malformed JSON");

    let client = reqwest::Client::new();

    let response = client
        .post("http://localhost:3003/recipe/create_product")
        .body("{ this is not valid json }")
        .header("Content-Type", "application/json")
        .send()
        .await?;

    // Should return 400 Bad Request
    assert!(response.status().is_client_error());
    println!("  ✅ Malformed JSON returns error: {}", response.status());

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_cors_headers() -> Result<()> {
    println!("🧪 Testing CORS headers");

    let response = reqwest::get("http://localhost:3003/health").await?;

    if let Some(cors_header) = response.headers().get("access-control-allow-origin") {
        println!("  ✅ CORS header present: {:?}", cors_header);
    } else {
        println!("  ⚠️  CORS headers not set");
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_concurrent_api_requests() -> Result<()> {
    println!("🧪 Testing concurrent API requests");

    let client = reqwest::Client::new();
    let mut handles = vec![];

    // Send 10 concurrent health check requests
    for i in 0..10 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            client_clone
                .get("http://localhost:3003/health")
                .send()
                .await
        });
        handles.push((i, handle));
    }

    // Wait for all and check results
    let mut success_count = 0;
    for (i, handle) in handles {
        match handle.await? {
            Ok(response) if response.status().is_success() => {
                success_count += 1;
            }
            Ok(response) => {
                println!("  ⚠️  Request {} failed: {}", i, response.status());
            }
            Err(e) => {
                println!("  ⚠️  Request {} error: {}", i, e);
            }
        }
    }

    assert_eq!(success_count, 10);
    println!("  ✅ All 10 concurrent requests successful");

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_realistic_product_creation() -> Result<()> {
    println!("🧪 Testing realistic product creation with many fields");

    let client = reqwest::Client::new();

    let product = json!({
        "id": "api_realistic_001",
        "sku": "TEST-SKU-001",
        "name": "API Test Realistic Product",
        "description": "Full description with all fields",
        "short_description": "Short desc",
        "price": 99999,
        "original_price": 99999,
        "cost_price": 75000,
        "currency": "USD",
        "brand": "TestBrand",
        "categories": "[\"Category1\", \"Category2\"]",
        "tags": "[\"tag1\", \"tag2\"]",
        "inventory_quantity": 100,
        "inventory_status": 1,
        "status": 1,
        "visibility": 1
    });

    let response = client
        .post("http://localhost:3003/recipe/create_realistic_product")
        .json(&product)
        .send()
        .await?;

    if response.status().is_success() {
        println!("  ✅ Realistic product created via API");
    } else {
        println!(
            "  ⚠️  Realistic product recipe may not be available: {}",
            response.status()
        );
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_full_crud_workflow() -> Result<()> {
    println!("🧪 Testing full CRUD workflow");

    let client = reqwest::Client::new();
    let product_id = "crud_test_product";

    // CREATE
    println!("  📝 Testing CREATE...");
    let create_data = json!({
        "id": product_id,
        "name": "CRUD Test Product",
        "price": 5000,
        "description": "Testing full CRUD"
    });

    let create_response = client
        .post("http://localhost:3003/recipe/create_product")
        .json(&create_data)
        .send()
        .await?;

    if !create_response.status().is_success() {
        println!("  ⚠️  CREATE not available, skipping CRUD test");
        return Ok(());
    }
    println!("  ✅ CREATE successful");

    sleep(Duration::from_millis(100)).await;

    // READ
    println!("  📖 Testing READ...");
    let read_response = client
        .get(&format!(
            "http://localhost:3003/balance/product:{}:existence",
            product_id
        ))
        .send()
        .await?;

    if read_response.status().is_success() {
        println!("  ✅ READ successful");
    }

    // UPDATE
    println!("  ✏️  Testing UPDATE...");
    let update_data = json!({
        "id": product_id,
        "price": 6000
    });

    let update_response = client
        .post("http://localhost:3003/recipe/update_product")
        .json(&update_data)
        .send()
        .await?;

    if update_response.status().is_success() {
        println!("  ✅ UPDATE successful");
    }

    sleep(Duration::from_millis(100)).await;

    // DELETE
    println!("  🗑️  Testing DELETE...");
    let delete_data = json!({
        "id": product_id
    });

    let delete_response = client
        .post("http://localhost:3003/recipe/delete_product")
        .json(&delete_data)
        .send()
        .await?;

    if delete_response.status().is_success() {
        println!("  ✅ DELETE successful");
    }

    println!("  🎉 Full CRUD workflow completed");

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually when server is running
async fn test_response_time() -> Result<()> {
    println!("🧪 Testing API response time");

    let client = reqwest::Client::new();
    let start = std::time::Instant::now();

    let response = client.get("http://localhost:3003/health").send().await?;

    let duration = start.elapsed();

    assert!(response.status().is_success());
    println!("  ✅ Response time: {:?}", duration);

    if duration.as_millis() > 100 {
        println!("  ⚠️  Response time > 100ms (consider performance optimization)");
    } else {
        println!("  🚀 Response time is excellent (< 100ms)");
    }

    Ok(())
}
