#!/usr/bin/env bash

# 🦖 ZIK_ZAK Comprehensive Test Runner
# Runs all test suites with proper reporting

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Emoji for fun
ROCKET="🚀"
CHECK="✅"
CROSS="❌"
WARNING="⚠️"
DINO="🦖"
TEST="🧪"
FIRE="🔥"
CHART="📊"

echo ""
echo -e "${DINO} ${FIRE} ${PURPLE}===========================================  ${NC}"
echo -e "${DINO} ${FIRE} ${PURPLE}  ZIK_ZAK COMPREHENSIVE TEST SUITE${NC}"
echo -e "${DINO} ${FIRE} ${PURPLE}  Testing the Revolution${NC}"
echo -e "${DINO} ${FIRE} ${PURPLE}===========================================${NC}"
echo ""

# Track results
TESTS_PASSED=0
TESTS_FAILED=0
START_TIME=$(date +%s)

# Function to run a test suite
run_test_suite() {
    local name=$1
    local command=$2
    local emoji=$3

    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${emoji} ${CYAN}Running: ${name}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    if eval "$command"; then
        echo ""
        echo -e "${CHECK} ${GREEN}${name} PASSED${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo ""
        echo -e "${CROSS} ${RED}${name} FAILED${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Parse arguments
RUN_ALL=true
RUN_UNIT=false
RUN_INTEGRATION=false
RUN_E2E=false
RUN_PERFORMANCE=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit)
            RUN_ALL=false
            RUN_UNIT=true
            shift
            ;;
        --integration)
            RUN_ALL=false
            RUN_INTEGRATION=true
            shift
            ;;
        --e2e)
            RUN_ALL=false
            RUN_E2E=true
            shift
            ;;
        --performance)
            RUN_ALL=false
            RUN_PERFORMANCE=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --unit           Run unit tests only"
            echo "  --integration    Run integration tests only"
            echo "  --e2e           Run end-to-end tests only"
            echo "  --performance   Run performance tests only"
            echo "  --verbose, -v   Verbose output"
            echo "  --help, -h      Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                    # Run all tests"
            echo "  $0 --unit            # Run only unit tests"
            echo "  $0 --unit --verbose  # Run unit tests with verbose output"
            exit 0
            ;;
        *)
            echo -e "${WARNING} Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set verbose flag
VERBOSE_FLAG=""
if [ "$VERBOSE" = true ]; then
    VERBOSE_FLAG="-- --nocapture"
fi

# 1. Code Quality Checks
if [ "$RUN_ALL" = true ]; then
    run_test_suite "Code Formatting Check" \
        "cargo fmt -- --check" \
        "✨"

    run_test_suite "Clippy Lints" \
        "cargo clippy --all-targets --all-features -- -D warnings" \
        "📋"
fi

# 2. Unit Tests
if [ "$RUN_ALL" = true ] || [ "$RUN_UNIT" = true ]; then
    run_test_suite "Unit Tests - SLED Storage" \
        "cargo test unit_sled_storage $VERBOSE_FLAG" \
        "${TEST}"

    run_test_suite "Unit Tests - Library Tests" \
        "cargo test --lib $VERBOSE_FLAG" \
        "${TEST}"
fi

# 3. Integration Tests
if [ "$RUN_ALL" = true ] || [ "$RUN_INTEGRATION" = true ]; then
    run_test_suite "Integration Tests - Spark & Recipes" \
        "cargo test integration_spark_recipes -- --test-threads=1 $VERBOSE_FLAG" \
        "🔗"

    run_test_suite "Integration Tests - Simple Concepts" \
        "cargo test --test simple_concepts_test $VERBOSE_FLAG" \
        "🔗"

    run_test_suite "Integration Tests - TigerBeetle" \
        "cargo test --test tigerbeetle_integration_test $VERBOSE_FLAG" \
        "🔗"

    run_test_suite "Integration Tests - ID Uniqueness" \
        "cargo test --test test_id_uniqueness $VERBOSE_FLAG" \
        "🔗"
fi

# 4. End-to-End Tests (if server is running)
if [ "$RUN_E2E" = true ]; then
    echo ""
    echo -e "${YELLOW}${WARNING} E2E tests require the server to be running${NC}"
    echo -e "${YELLOW}   Start server in another terminal: cargo run${NC}"
    read -p "Press Enter to continue with E2E tests, or Ctrl+C to skip..."

    run_test_suite "E2E API Tests" \
        "cargo test e2e_ -- --ignored $VERBOSE_FLAG" \
        "🌐"
fi

# 5. Performance Tests
if [ "$RUN_PERFORMANCE" = true ]; then
    echo ""
    echo -e "${YELLOW}${WARNING} Performance tests take longer and run in release mode${NC}"
    read -p "Press Enter to continue with performance tests, or Ctrl+C to skip..."

    run_test_suite "Performance Tests" \
        "cargo test --release performance_ -- --ignored $VERBOSE_FLAG" \
        "${CHART}"
fi

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

# Print summary
echo ""
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${DINO} ${PURPLE}TEST SUMMARY${NC}"
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}${CHECK} Tests Passed: ${TESTS_PASSED}${NC}"

if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}${CROSS} Tests Failed: ${TESTS_FAILED}${NC}"
else
    echo -e "${GREEN}${CHECK} Tests Failed: 0${NC}"
fi

echo ""
echo -e "${CYAN}⏱️  Total Duration: ${DURATION}s${NC}"
echo ""

# Final message
if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}${CHECK} ${DINO} ${FIRE} ALL TESTS PASSED! ${FIRE} ${DINO} ${CHECK}${NC}"
    echo -e "${GREEN}Backend development is still DEAD! 💀${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}${CROSS} Some tests failed. Review output above.${NC}"
    echo ""
    exit 1
fi
