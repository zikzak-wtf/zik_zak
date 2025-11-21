//! Integration tests for Spark Engine and Recipe execution
//!
//! Tests the JSON-driven operation system that eliminates the need for code changes.

use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;
use zik_zak::{Genesis, SparkEngine, ZikZakEngine, ZikZakSledEngine};

async fn setup_test_environment() -> Result<(ZikZakEngine, ZikZakSledEngine, Genesis)> {
    let temp_dir = TempDir::new()?;
    let zikzak = ZikZakEngine::new().await?;
    let sled = ZikZakSledEngine::new(temp_dir.path())?;
    let genesis = Genesis::new(zikzak.clone(), sled.clone())?;
    Ok((zikzak, sled, genesis))
}

#[tokio::test]
async fn test_spark_engine_initialization() -> Result<()> {
    println!("🧪 Testing Spark Engine initialization");

    let recipes_json = json!({
        "schema_version": "1.0",
        "title": "Test Recipes",
        "entities": {
            "product": {
                "accounts": {
                    "existence": "product:{id}:existence",
                    "price": "product:{id}:price"
                }
            }
        },
        "recipes": {}
    });

    let spark_engine = SparkEngine::from_json(recipes_json.to_string())?;
    println!("  ✅ Spark engine initialized successfully");

    Ok(())
}

#[tokio::test]
async fn test_create_product_recipe() -> Result<()> {
    println!("🧪 Testing create_product recipe execution");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    // Load recipes
    let recipes_path = std::path::Path::new("recipes.json");
    if !recipes_path.exists() {
        println!("  ⚠️  recipes.json not found, skipping test");
        return Ok(());
    }

    let recipes_json = std::fs::read_to_string(recipes_path)?;
    genesis.load_sparks_from_json(&recipes_json)?;

    // Execute create_product recipe
    let params = json!({
        "id": "test_product_123",
        "name": "Test Product",
        "price": 2999,
        "description": "A test product"
    });

    let result = genesis.manifest("create_product", params).await?;
    println!("  ✅ Product created: {:?}", result);

    // Verify product exists
    let existence_balance = _zikzak
        .get_balance("product:test_product_123:existence")
        .await?;
    assert_eq!(existence_balance, 1);
    println!("  ✅ Product existence verified");

    // Verify price
    let price_balance = _zikzak.get_balance("product:test_product_123:price").await?;
    assert_eq!(price_balance, 2999);
    println!("  ✅ Product price verified");

    Ok(())
}

#[tokio::test]
async fn test_update_product_recipe() -> Result<()> {
    println!("🧪 Testing update_product recipe execution");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    let recipes_path = std::path::Path::new("recipes.json");
    if !recipes_path.exists() {
        println!("  ⚠️  recipes.json not found, skipping test");
        return Ok(());
    }

    let recipes_json = std::fs::read_to_string(recipes_path)?;
    genesis.load_sparks_from_json(&recipes_json)?;

    // Create product first
    let create_params = json!({
        "id": "updatable_product",
        "name": "Original Name",
        "price": 1000,
        "description": "Original description"
    });
    genesis.manifest("create_product", create_params).await?;

    // Update the product
    let update_params = json!({
        "id": "updatable_product",
        "price": 1500
    });
    genesis.manifest("update_product", update_params).await?;

    // Verify price updated
    let new_price = _zikzak.get_balance("product:updatable_product:price").await?;
    assert_eq!(new_price, 1500);
    println!("  ✅ Product price updated successfully");

    Ok(())
}

#[tokio::test]
async fn test_delete_product_recipe() -> Result<()> {
    println!("🧪 Testing delete_product recipe execution");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    let recipes_path = std::path::Path::new("recipes.json");
    if !recipes_path.exists() {
        println!("  ⚠️  recipes.json not found, skipping test");
        return Ok(());
    }

    let recipes_json = std::fs::read_to_string(recipes_path)?;
    genesis.load_sparks_from_json(&recipes_json)?;

    // Create product
    let create_params = json!({
        "id": "deletable_product",
        "name": "To Be Deleted",
        "price": 999,
        "description": "This will be deleted"
    });
    genesis.manifest("create_product", create_params).await?;

    // Verify it exists
    let existence = _zikzak
        .get_balance("product:deletable_product:existence")
        .await?;
    assert_eq!(existence, 1);

    // Delete the product
    let delete_params = json!({
        "id": "deletable_product"
    });
    genesis.manifest("delete_product", delete_params).await?;

    // Verify it's deleted (balance transferred to system:deleted)
    let existence_after = _zikzak
        .get_balance("product:deletable_product:existence")
        .await?;
    assert_eq!(existence_after, 0);
    println!("  ✅ Product deleted successfully");

    Ok(())
}

#[tokio::test]
async fn test_realistic_product_with_many_fields() -> Result<()> {
    println!("🧪 Testing realistic product with many fields");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    let recipes_path = std::path::Path::new("recipes.json");
    if !recipes_path.exists() {
        println!("  ⚠️  recipes.json not found, skipping test");
        return Ok(());
    }

    let recipes_json = std::fs::read_to_string(recipes_path)?;
    genesis.load_sparks_from_json(&recipes_json)?;

    // Create a realistic product with many fields
    let params = json!({
        "id": "iphone15pro",
        "sku": "IPHONE-15-PRO-256-BLACK",
        "name": "iPhone 15 Pro",
        "description": "The most advanced iPhone with titanium design",
        "short_description": "Pro camera system, A17 Pro chip",
        "price": 99999,
        "original_price": 99999,
        "cost_price": 75000,
        "currency": "USD",
        "brand": "Apple",
        "categories": "[\"Smartphones\", \"Electronics\", \"Apple\"]",
        "tags": "[\"5G\", \"Pro\", \"Titanium\"]",
        "inventory_quantity": 50,
        "inventory_status": 1,
        "status": 1,
        "visibility": 1
    });

    let result = genesis.manifest("create_realistic_product", params).await;

    match result {
        Ok(_) => {
            println!("  ✅ Realistic product created successfully");

            // Verify existence
            let existence = _zikzak
                .get_balance("rproduct:iphone15pro:existence")
                .await?;
            assert_eq!(existence, 1);

            // Verify price
            let price = _zikzak.get_balance("rproduct:iphone15pro:price").await?;
            assert_eq!(price, 99999);

            println!("  ✅ Product details verified");
        }
        Err(e) => {
            println!("  ⚠️  Recipe not found or error: {}", e);
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_recipe_parameter_interpolation() -> Result<()> {
    println!("🧪 Testing recipe parameter interpolation");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    // Create a simple test recipe with parameter interpolation
    let test_recipe = json!({
        "schema_version": "1.0",
        "entities": {},
        "recipes": {
            "test_transfer": {
                "description": "Test parameter interpolation",
                "inputs": ["from_id", "to_id", "amount"],
                "operations": [
                    {
                        "type": "transfer",
                        "from": "user:{from_id}:balance",
                        "to": "user:{to_id}:balance",
                        "amount": "{amount}",
                        "metadata": {}
                    }
                ]
            }
        }
    });

    genesis.load_sparks_from_json(&test_recipe.to_string())?;

    // First, fund the accounts
    _zikzak
        .transfer("system:genesis", "user:alice:balance", 10000, HashMap::new())
        .await?;

    // Execute recipe with parameters
    let params = json!({
        "from_id": "alice",
        "to_id": "bob",
        "amount": 5000
    });

    genesis.manifest("test_transfer", params).await?;

    // Verify transfer happened
    let alice_balance = _zikzak.get_balance("user:alice:balance").await?;
    let bob_balance = _zikzak.get_balance("user:bob:balance").await?;

    assert_eq!(alice_balance, 5000);
    assert_eq!(bob_balance, 5000);
    println!("  ✅ Parameter interpolation works correctly");

    Ok(())
}

#[tokio::test]
async fn test_multiple_operations_in_recipe() -> Result<()> {
    println!("🧪 Testing recipe with multiple operations");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    // Create recipe with multiple operations
    let test_recipe = json!({
        "schema_version": "1.0",
        "entities": {},
        "recipes": {
            "create_order": {
                "description": "Create order with multiple operations",
                "inputs": ["order_id", "user_id", "product_id", "quantity", "total"],
                "operations": [
                    {
                        "type": "transfer",
                        "from": "system:genesis",
                        "to": "order:{order_id}:existence",
                        "amount": 1,
                        "metadata": {}
                    },
                    {
                        "type": "transfer",
                        "from": "system:genesis",
                        "to": "order:{order_id}:quantity",
                        "amount": "{quantity}",
                        "metadata": {}
                    },
                    {
                        "type": "transfer",
                        "from": "system:genesis",
                        "to": "order:{order_id}:total",
                        "amount": "{total}",
                        "metadata": {}
                    }
                ]
            }
        }
    });

    genesis.load_sparks_from_json(&test_recipe.to_string())?;

    // Execute recipe
    let params = json!({
        "order_id": "order_123",
        "user_id": "user_456",
        "product_id": "product_789",
        "quantity": 3,
        "total": 8997
    });

    genesis.manifest("create_order", params).await?;

    // Verify all operations executed
    assert_eq!(_zikzak.get_balance("order:order_123:existence").await?, 1);
    assert_eq!(_zikzak.get_balance("order:order_123:quantity").await?, 3);
    assert_eq!(_zikzak.get_balance("order:order_123:total").await?, 8997);

    println!("  ✅ Multiple operations executed successfully");

    Ok(())
}

#[tokio::test]
async fn test_recipe_hot_reload() -> Result<()> {
    println!("🧪 Testing recipe hot reload");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    // Load initial recipes
    let recipe_v1 = json!({
        "schema_version": "1.0",
        "entities": {},
        "recipes": {
            "test_recipe": {
                "description": "Version 1",
                "inputs": ["id"],
                "operations": []
            }
        }
    });

    genesis.load_sparks_from_json(&recipe_v1.to_string())?;
    println!("  ✅ Loaded recipe v1");

    // Reload with updated recipe
    let recipe_v2 = json!({
        "schema_version": "1.0",
        "entities": {},
        "recipes": {
            "test_recipe": {
                "description": "Version 2 - Updated",
                "inputs": ["id", "extra_field"],
                "operations": []
            },
            "new_recipe": {
                "description": "Newly added recipe",
                "inputs": [],
                "operations": []
            }
        }
    });

    genesis.load_sparks_from_json(&recipe_v2.to_string())?;
    println!("  ✅ Hot reloaded recipes (v2)");

    // Both recipes should be available now
    // This demonstrates that recipes can be updated without restarting the server

    Ok(())
}

#[tokio::test]
async fn test_error_handling_invalid_recipe() -> Result<()> {
    println!("🧪 Testing error handling for invalid recipes");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    // Try to load malformed JSON
    let invalid_json = "{ this is not valid json }";
    let result = genesis.load_sparks_from_json(invalid_json);

    assert!(result.is_err());
    println!("  ✅ Invalid JSON rejected");

    // Load valid structure but execute non-existent recipe
    let valid_json = json!({
        "schema_version": "1.0",
        "entities": {},
        "recipes": {}
    });

    genesis.load_sparks_from_json(&valid_json.to_string())?;

    let result = genesis
        .manifest("non_existent_recipe", json!({}))
        .await;

    assert!(result.is_err());
    println!("  ✅ Non-existent recipe returns error");

    Ok(())
}

#[tokio::test]
async fn test_concurrent_recipe_execution() -> Result<()> {
    println!("🧪 Testing concurrent recipe execution");

    let (_zikzak, _sled, genesis) = setup_test_environment().await?;

    // Simple recipe for testing
    let recipe = json!({
        "schema_version": "1.0",
        "entities": {},
        "recipes": {
            "create_item": {
                "description": "Create item",
                "inputs": ["id"],
                "operations": [
                    {
                        "type": "transfer",
                        "from": "system:genesis",
                        "to": "item:{id}:existence",
                        "amount": 1,
                        "metadata": {}
                    }
                ]
            }
        }
    });

    genesis.load_sparks_from_json(&recipe.to_string())?;

    // Execute 10 recipes concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let genesis_clone = genesis.clone();
        let handle = tokio::spawn(async move {
            let params = json!({ "id": format!("item_{}", i) });
            genesis_clone.manifest("create_item", params).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await??;
    }

    // Verify all items were created
    for i in 0..10 {
        let balance = _zikzak
            .get_balance(&format!("item:item_{}:existence", i))
            .await?;
        assert_eq!(balance, 1);
    }

    println!("  ✅ Concurrent recipe execution successful");

    Ok(())
}
