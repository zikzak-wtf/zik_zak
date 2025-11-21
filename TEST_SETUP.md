# 🦖 ZIK_ZAK Test Setup Guide

## Prerequisites

Before running tests, you need to set up the TigerBeetle dependency.

### 1. Clone TigerBeetle

```bash
cd /home/user/zik_zak
git clone https://github.com/tigerbeetle/tigerbeetle.git tigerbeetle-repo
```

### 2. Build TigerBeetle Rust Client

```bash
cd tigerbeetle-repo
# Follow TigerBeetle's build instructions for the Rust client
# This typically involves building the main TigerBeetle project first
```

### 3. Verify Setup

```bash
cd zik_zak
./validate_tests.sh
```

If you see compilation errors, ensure:
- TigerBeetle repo is cloned at `../tigerbeetle-repo/`
- Rust client is built at `src/clients/rust/`
- Path in `Cargo.toml` matches: `path = "../tigerbeetle-repo/src/clients/rust"`

### 4. Run Tests

Once setup is complete:

```bash
# Quick validation
./validate_tests.sh

# Run all tests
./run_all_tests.sh

# Or use cargo directly
cargo test
```

## Alternative: Mock Tests

If TigerBeetle setup is not available, you can:

1. **Run unit tests that don't require TigerBeetle:**
   ```bash
   cargo test unit_sled_storage
   ```

2. **Use the existing test structure:**
   The project has integration tests that work with TigerBeetle when available.

## Troubleshooting

### "Unable to update tigerbeetle-repo"

**Problem:** TigerBeetle repository not found

**Solution:**
```bash
cd /home/user/zik_zak
git clone https://github.com/tigerbeetle/tigerbeetle.git tigerbeetle-repo
cd tigerbeetle-repo
# Build according to TigerBeetle documentation
```

### "Tests are slow"

**Problem:** Tests taking too long

**Solution:**
```bash
# Run tests in parallel (default)
cargo test

# Run unit tests only (faster)
./run_all_tests.sh --unit

# Run specific test
cargo test test_name
```

### "TigerBeetle connection failed"

**Problem:** TigerBeetle server not running

**Solution:**
Integration and E2E tests create ephemeral TigerBeetle instances. Ensure:
- TigerBeetle is properly built
- No port conflicts
- Sufficient permissions

## Test Categories

Tests are organized by dependency requirements:

### No External Dependencies
- `unit_sled_storage` - SLED tests (uses temp dirs only)
- Library tests - Pure Rust logic

### Requires TigerBeetle Build
- `integration_spark_recipes` - Recipe execution
- `tigerbeetle_integration_test` - TigerBeetle integration
- Other integration tests

### Requires Running Server
- `e2e_api_tests` - HTTP API tests (marked with `#[ignore]`)

### Manual Execution
- `performance_tests` - Benchmarks (marked with `#[ignore]`)

## Quick Start (No Setup)

If you just want to validate the test code without running:

```bash
# Check test code syntax
grep -r "#\[tokio::test\]" tests/

# Count tests
find tests -name "*.rs" | wc -l

# View test structure
ls -la tests/
```

## CI/CD Note

The GitHub Actions CI/CD pipeline handles TigerBeetle setup automatically:
- Clones dependencies
- Builds required components
- Runs all test suites
- Reports results

See `.github/workflows/test.yml` for details.
