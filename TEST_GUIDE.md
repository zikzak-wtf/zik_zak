# 🦖 ZIK_ZAK Testing Guide

## Overview

This guide covers how to run, write, and maintain tests for the ZIK_ZAK revolutionary accounting-based backend system.

## Quick Start

```bash
cd zik_zak

# Run all tests
./run_all_tests.sh

# Run specific test categories
cargo test unit_           # Unit tests only
cargo test integration_    # Integration tests only
cargo test e2e_           # End-to-end tests only
```

## Test Categories

### 1. Unit Tests

**Location:** `tests/unit_*.rs`

**Purpose:** Test individual components in isolation

**Run:**
```bash
cargo test unit_ --lib
```

**Coverage:**
- ✅ SLED storage operations (`unit_sled_storage.rs`)
- ✅ ZikZak engine primitives
- ✅ TigerBeetle client
- ✅ Spark engine
- ✅ Genesis operations

**Example:**
```rust
#[tokio::test]
async fn test_sled_store_basic_operations() -> Result<()> {
    let store = SledVarCharStore::new(temp_path)?;
    store.set("key", "value")?;
    assert_eq!(store.get("key")?, Some("value".to_string()));
    Ok(())
}
```

### 2. Integration Tests

**Location:** `tests/integration_*.rs`

**Purpose:** Test component interactions and recipe execution

**Run:**
```bash
cargo test integration_ -- --test-threads=1
```

**Coverage:**
- ✅ Spark engine + recipe execution (`integration_spark_recipes.rs`)
- ✅ SLED + TigerBeetle coordination
- ✅ Recipe hot-reloading
- ✅ Multi-operation workflows

**Example:**
```rust
#[tokio::test]
async fn test_create_product_recipe() -> Result<()> {
    let genesis = Genesis::new(zikzak, sled)?;
    genesis.load_sparks_from_json(&recipes_json)?;

    let params = json!({
        "id": "product_123",
        "name": "Test Product",
        "price": 2999
    });

    genesis.manifest("create_product", params).await?;
    Ok(())
}
```

### 3. End-to-End API Tests

**Location:** `tests/e2e_*.rs`

**Purpose:** Test the complete HTTP API surface

**Run:**
```bash
# Start server in one terminal
cargo run

# Run E2E tests in another terminal
cargo test e2e_ -- --ignored
```

**Coverage:**
- ✅ Health endpoints (`e2e_api_tests.rs`)
- ✅ Recipe execution via REST
- ✅ Balance queries
- ✅ Error handling
- ✅ CORS
- ✅ Concurrent requests
- ✅ Full CRUD workflows

**Example:**
```rust
#[tokio::test]
#[ignore] // Requires running server
async fn test_create_product_via_api() -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3003/recipe/create_product")
        .json(&product_data)
        .send()
        .await?;

    assert!(response.status().is_success());
    Ok(())
}
```

### 4. Performance Tests

**Location:** `tests/performance_*.rs`

**Purpose:** Measure throughput, latency, and resource usage

**Run:**
```bash
cargo test --release performance_ -- --ignored --nocapture
```

**Coverage:**
- ✅ Transfer throughput (`performance_tests.rs`)
- ✅ Balance query performance
- ✅ Concurrent transfer load
- ✅ Latency distribution (p50, p95, p99)
- ✅ Sustained load testing
- ✅ Memory usage

**Example:**
```rust
#[tokio::test]
#[ignore]
async fn test_transfer_throughput() -> Result<()> {
    let start = Instant::now();
    for i in 0..1000 {
        engine.transfer(...).await?;
    }
    let tps = 1000.0 / start.elapsed().as_secs_f64();
    println!("Throughput: {:.2} transfers/sec", tps);
    Ok(())
}
```

## Test Execution Patterns

### Running All Tests

```bash
# Standard test run
cargo test

# With output
cargo test -- --nocapture

# Single-threaded (for integration tests)
cargo test -- --test-threads=1

# Specific test
cargo test test_sled_store_basic_operations
```

### Running E2E Tests

```bash
# Terminal 1: Start server
cd zik_zak
cargo run

# Terminal 2: Run E2E tests
cargo test e2e_ -- --ignored --nocapture
```

### Running Performance Tests

```bash
# Run in release mode for accurate performance
cargo test --release performance_ -- --ignored --nocapture

# Run specific performance test
cargo test --release test_transfer_throughput -- --ignored --nocapture
```

## Writing New Tests

### Unit Test Template

```rust
use anyhow::Result;
use zik_zak::*;

#[tokio::test]
async fn test_my_feature() -> Result<()> {
    println!("🧪 Testing my feature");

    // Setup
    let engine = ZikZakEngine::new().await?;

    // Execute
    let result = engine.do_something().await?;

    // Verify
    assert_eq!(result, expected);
    println!("  ✅ Feature works correctly");

    Ok(())
}
```

### Integration Test Template

```rust
use anyhow::Result;
use serde_json::json;
use zik_zak::*;

#[tokio::test]
async fn test_recipe_workflow() -> Result<()> {
    println!("🧪 Testing recipe workflow");

    // Setup
    let genesis = setup_genesis().await?;
    genesis.load_sparks_from_json(&recipes)?;

    // Execute recipe
    let params = json!({"id": "test"});
    let result = genesis.manifest("recipe_name", params).await?;

    // Verify results
    assert!(result.is_ok());
    println!("  ✅ Workflow completed");

    Ok(())
}
```

### E2E Test Template

```rust
#[tokio::test]
#[ignore] // Requires running server
async fn test_api_endpoint() -> Result<()> {
    println!("🧪 Testing API endpoint");

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3003/endpoint")
        .json(&data)
        .send()
        .await?;

    assert!(response.status().is_success());
    println!("  ✅ API works");

    Ok(())
}
```

## Test Organization

```
tests/
├── unit_sled_storage.rs          # SLED unit tests
├── unit_zikzak_engine.rs         # ZikZak engine unit tests
├── unit_tigerbeetle_client.rs    # TigerBeetle unit tests
├── unit_spark_engine.rs          # Spark engine unit tests
├── unit_genesis.rs               # Genesis unit tests
├── integration_spark_recipes.rs  # Recipe integration tests
├── integration_dual_storage.rs   # SLED + TigerBeetle tests
├── e2e_api_tests.rs              # HTTP API tests
├── e2e_system.rs                 # Full system tests
├── performance_tests.rs          # Performance benchmarks
└── fixtures/                     # Test data and fixtures
```

## CI/CD Integration

Tests run automatically on:
- ✅ Push to main/develop
- ✅ Pull requests
- ✅ Manual trigger

**GitHub Actions Workflow:**
```yaml
jobs:
  quick-check:    # Format + Lint (30s)
  unit-tests:     # Unit tests (5min)
  integration:    # Integration tests (10min)
  build-test:     # Build verification (5min)
  performance:    # Benchmarks (main branch only)
```

**Status:** ![Tests](https://github.com/zikzak-wtf/zik_zak/workflows/test/badge.svg)

## Test Best Practices

### ✅ DO

1. **Use descriptive test names**
   ```rust
   test_sled_handles_unicode_correctly  // Good
   test_1                               // Bad
   ```

2. **Print progress in tests**
   ```rust
   println!("🧪 Testing feature X");
   println!("  ✅ Step 1 complete");
   ```

3. **Use `#[ignore]` for manual tests**
   ```rust
   #[tokio::test]
   #[ignore] // Requires running server
   async fn test_api() -> Result<()> { ... }
   ```

4. **Clean up test resources**
   ```rust
   let temp_dir = TempDir::new()?;  // Auto-cleanup
   ```

5. **Test both success and failure cases**
   ```rust
   // Success case
   assert!(engine.transfer(...).await.is_ok());

   // Failure case
   assert!(engine.transfer_invalid(...).await.is_err());
   ```

### ❌ DON'T

1. **Don't use hardcoded paths**
   ```rust
   // Bad
   let store = SledVarCharStore::new("/tmp/test")?;

   // Good
   let temp_dir = TempDir::new()?;
   let store = SledVarCharStore::new(temp_dir.path())?;
   ```

2. **Don't leave dangling resources**
   ```rust
   // Bad - manual cleanup needed
   std::fs::remove_dir_all("/tmp/test")?;

   // Good - automatic cleanup
   let _temp = TempDir::new()?;
   ```

3. **Don't make tests dependent on each other**
   ```rust
   // Bad - test_b depends on test_a
   // Good - each test is independent
   ```

4. **Don't skip verification**
   ```rust
   // Bad
   engine.transfer(...).await?;

   // Good
   engine.transfer(...).await?;
   assert_eq!(engine.get_balance(...).await?, expected);
   ```

## Troubleshooting

### Test Failures

**"TigerBeetle connection failed"**
```bash
# Ensure TigerBeetle is properly built
cd ../tigerbeetle-repo
# Follow TigerBeetle setup instructions
```

**"SLED database locked"**
```bash
# Run tests single-threaded
cargo test -- --test-threads=1
```

**"Server not running" (E2E tests)**
```bash
# Start server first
cargo run
# Then run E2E tests in another terminal
cargo test e2e_ -- --ignored
```

### Performance Issues

**Tests running slowly**
```bash
# Use release mode
cargo test --release

# Run fewer iterations
# Adjust num_transfers, num_queries in test code
```

**Memory leaks suspected**
```bash
# Run with memory profiler
cargo test --features memory-profiling
```

## Test Metrics

**Current Coverage Goals:**
- Overall: > 80%
- Critical paths (transfer, balance): > 95%
- Error handling: > 90%

**Performance Targets:**
- Transfer throughput: > 1000 TPS
- Balance queries: > 10,000 QPS
- p99 latency: < 50ms
- Concurrent operations: 100+ simultaneous

## Contributing Tests

When adding new features:

1. ✅ Write unit tests first
2. ✅ Add integration tests for workflows
3. ✅ Update E2E tests if API changes
4. ✅ Add performance tests for critical paths
5. ✅ Update this guide with new patterns

## Resources

- **Test Strategy:** `TESTING_STRATEGY.md`
- **Main README:** `README.md`
- **CI/CD Config:** `.github/workflows/test.yml`

---

**🦖 Testing the revolution, one transfer at a time.**

*Questions? Open an issue or PR!*
