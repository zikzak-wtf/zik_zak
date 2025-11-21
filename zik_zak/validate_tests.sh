#!/usr/bin/env bash

# 🧪 Quick Test Validation Script
# Checks if all tests compile without running them

set -e

echo "🧪 Validating ZIK_ZAK test suite..."
echo ""

echo "📦 Checking test compilation..."
if cargo test --no-run; then
    echo "✅ All tests compile successfully"
else
    echo "❌ Test compilation failed"
    exit 1
fi

echo ""
echo "📋 Test summary:"
echo ""

# Count test files
UNIT_TESTS=$(find tests -name "unit_*.rs" 2>/dev/null | wc -l)
INTEGRATION_TESTS=$(find tests -name "integration_*.rs" 2>/dev/null | wc -l)
E2E_TESTS=$(find tests -name "e2e_*.rs" 2>/dev/null | wc -l)
PERF_TESTS=$(find tests -name "performance_*.rs" 2>/dev/null | wc -l)

echo "  Unit test files: $UNIT_TESTS"
echo "  Integration test files: $INTEGRATION_TESTS"
echo "  E2E test files: $E2E_TESTS"
echo "  Performance test files: $PERF_TESTS"
echo ""

# List all test functions
echo "📊 Test function count:"
TOTAL_TESTS=$(grep -r "#\[tokio::test\]" tests/ 2>/dev/null | wc -l)
echo "  Total test functions: $TOTAL_TESTS"
echo ""

echo "✅ Test validation complete!"
echo ""
echo "To run tests:"
echo "  ./run_all_tests.sh           # Run all tests"
echo "  ./run_all_tests.sh --unit    # Run unit tests only"
echo "  cargo test                   # Standard cargo test"
