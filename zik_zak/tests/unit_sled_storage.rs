//! Unit tests for SLED varchar storage
//!
//! Tests the embedded key-value store that handles text data
//! alongside TigerBeetle's numeric accounting.

use anyhow::Result;
use tempfile::TempDir;
use zik_zak::sled::{SledVarCharStore, ZikZakSledEngine};

#[tokio::test]
async fn test_sled_store_basic_operations() -> Result<()> {
    println!("🧪 Testing basic SLED store operations");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    // Test set and get
    let key = "product:123:name";
    let value = "iPhone 15 Pro";

    store.set(key, value)?;
    let retrieved = store.get(key)?;

    assert_eq!(retrieved, Some(value.to_string()));
    println!("  ✅ Set and get operations work");

    // Test non-existent key
    let missing = store.get("product:999:name")?;
    assert_eq!(missing, None);
    println!("  ✅ Non-existent keys return None");

    // Test overwrite
    let new_value = "iPhone 15 Pro Max";
    store.set(key, new_value)?;
    let retrieved = store.get(key)?;
    assert_eq!(retrieved, Some(new_value.to_string()));
    println!("  ✅ Overwrite works correctly");

    Ok(())
}

#[tokio::test]
async fn test_sled_delete_operations() -> Result<()> {
    println!("🧪 Testing SLED delete operations");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    let key = "user:456:email";
    let value = "user@example.com";

    store.set(key, value)?;
    assert!(store.get(key)?.is_some());

    // Delete the key
    store.delete(key)?;
    assert_eq!(store.get(key)?, None);
    println!("  ✅ Delete operation works");

    // Delete non-existent key should not error
    store.delete("non:existent:key")?;
    println!("  ✅ Deleting non-existent key is safe");

    Ok(())
}

#[tokio::test]
async fn test_sled_unicode_handling() -> Result<()> {
    println!("🧪 Testing Unicode text storage");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    let test_cases = vec![
        ("emoji", "🦖💀🔥⚡"),
        ("chinese", "中文测试"),
        ("japanese", "日本語テスト"),
        ("arabic", "اختبار عربي"),
        ("mixed", "Hello 世界! 🌍"),
    ];

    for (name, text) in test_cases {
        let key = format!("test:{}:text", name);
        store.set(&key, text)?;
        let retrieved = store.get(&key)?;
        assert_eq!(retrieved, Some(text.to_string()));
        println!("  ✅ {} text: {}", name, text);
    }

    Ok(())
}

#[tokio::test]
async fn test_sled_large_text_storage() -> Result<()> {
    println!("🧪 Testing large text storage");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    // Test 1KB text
    let small_text = "a".repeat(1024);
    store.set("test:small", &small_text)?;
    assert_eq!(store.get("test:small")?, Some(small_text));
    println!("  ✅ 1KB text stored");

    // Test 10KB text
    let medium_text = "b".repeat(10 * 1024);
    store.set("test:medium", &medium_text)?;
    assert_eq!(store.get("test:medium")?, Some(medium_text));
    println!("  ✅ 10KB text stored");

    // Test 100KB text
    let large_text = "c".repeat(100 * 1024);
    store.set("test:large", &large_text)?;
    assert_eq!(store.get("test:large")?, Some(large_text));
    println!("  ✅ 100KB text stored");

    Ok(())
}

#[tokio::test]
async fn test_sled_persistence() -> Result<()> {
    println!("🧪 Testing SLED persistence");

    let temp_dir = TempDir::new()?;
    let path = temp_dir.path();

    // Create store and write data
    {
        let store = SledVarCharStore::new(path)?;
        store.set("product:123:name", "iPhone")?;
        store.set("product:456:name", "MacBook")?;
        // Store goes out of scope and should flush
    }

    // Reopen and verify data persists
    {
        let store = SledVarCharStore::new(path)?;
        assert_eq!(store.get("product:123:name")?, Some("iPhone".to_string()));
        assert_eq!(store.get("product:456:name")?, Some("MacBook".to_string()));
        println!("  ✅ Data persists across reopens");
    }

    Ok(())
}

#[tokio::test]
async fn test_sled_concurrent_writes() -> Result<()> {
    println!("🧪 Testing concurrent SLED writes");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let store_clone = store.clone();
            tokio::spawn(async move {
                let key = format!("product:{}:name", i);
                let value = format!("Product {}", i);
                store_clone.set(&key, &value).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    // Verify all writes succeeded
    for i in 0..10 {
        let key = format!("product:{}:name", i);
        let expected = format!("Product {}", i);
        assert_eq!(store.get(&key)?, Some(expected));
    }

    println!("  ✅ Concurrent writes successful");

    Ok(())
}

#[tokio::test]
async fn test_sled_engine_with_hash() -> Result<()> {
    println!("🧪 Testing SLED engine with hash-based keys");

    let temp_dir = TempDir::new()?;
    let engine = ZikZakSledEngine::new(temp_dir.path())?;

    // Store text with automatic hashing
    let key = "product:iphone15:description";
    let text = "The most advanced iPhone ever with A17 Pro chip";

    engine.set_text(key, text)?;
    let retrieved = engine.get_text(key)?;

    assert_eq!(retrieved, Some(text.to_string()));
    println!("  ✅ Hash-based storage works");

    // Test with JSON data
    let json_key = "product:iphone15:specs";
    let json_text = r#"{"screen": "6.1 inch", "storage": "256GB", "ram": "8GB"}"#;

    engine.set_text(json_key, json_text)?;
    let retrieved_json = engine.get_text(json_key)?;

    assert_eq!(retrieved_json, Some(json_text.to_string()));
    println!("  ✅ JSON storage works");

    Ok(())
}

#[tokio::test]
async fn test_sled_batch_operations() -> Result<()> {
    println!("🧪 Testing batch operations");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    // Write multiple items
    let items = vec![
        ("product:1:name", "Item 1"),
        ("product:2:name", "Item 2"),
        ("product:3:name", "Item 3"),
        ("product:4:name", "Item 4"),
        ("product:5:name", "Item 5"),
    ];

    for (key, value) in &items {
        store.set(key, value)?;
    }

    // Verify all items
    for (key, value) in &items {
        assert_eq!(store.get(key)?, Some(value.to_string()));
    }

    println!("  ✅ Batch writes successful");

    Ok(())
}

#[tokio::test]
async fn test_sled_special_characters() -> Result<()> {
    println!("🧪 Testing special characters in keys and values");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    // Test colons in values (keys already have colons)
    store.set("test:1", "value:with:colons")?;
    assert_eq!(store.get("test:1")?, Some("value:with:colons".to_string()));

    // Test quotes and special chars
    let special_value = r#"Special "quotes" and 'apostrophes' & symbols!"#;
    store.set("test:2", special_value)?;
    assert_eq!(store.get("test:2")?, Some(special_value.to_string()));

    // Test newlines
    let multiline = "Line 1\nLine 2\nLine 3";
    store.set("test:3", multiline)?;
    assert_eq!(store.get("test:3")?, Some(multiline.to_string()));

    println!("  ✅ Special characters handled correctly");

    Ok(())
}

#[tokio::test]
async fn test_sled_empty_values() -> Result<()> {
    println!("🧪 Testing empty value handling");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    // Store empty string
    store.set("test:empty", "")?;
    let retrieved = store.get("test:empty")?;
    assert_eq!(retrieved, Some(String::new()));
    println!("  ✅ Empty strings stored correctly");

    Ok(())
}

#[tokio::test]
async fn test_sled_count_and_metrics() -> Result<()> {
    println!("🧪 Testing count and metrics");

    let temp_dir = TempDir::new()?;
    let store = SledVarCharStore::new(temp_dir.path())?;

    // Write some data
    for i in 0..100 {
        store.set(&format!("item:{}", i), &format!("value {}", i))?;
    }

    println!("  ✅ Stored 100 items");

    // Verify retrieval works
    for i in 0..100 {
        let key = format!("item:{}", i);
        assert!(store.get(&key)?.is_some());
    }

    println!("  ✅ All 100 items retrievable");

    Ok(())
}
