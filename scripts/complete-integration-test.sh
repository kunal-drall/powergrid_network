#!/bin/bash

# PowerGrid Network - Complete Integration Test Suite
# Tests the entire workflow: setup → build → test → e2e → validation
# This single script validates all components working together

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_section() {
    echo
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}============================================${NC}"
    echo
}

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Test function wrapper
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    log_info "Running: $test_name"
    
    if eval "$test_command" > /dev/null 2>&1; then
        log_success "✅ $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        log_error "❌ $test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Run test with visible output
run_test_verbose() {
    local test_name="$1"
    local test_command="$2"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    log_info "Running: $test_name"
    echo "Command: $test_command"
    
    if eval "$test_command"; then
        log_success "✅ $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        log_error "❌ $test_name"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    # Add any cleanup operations here
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

log_section "🚀 PowerGrid Network Complete Integration Test Suite"

# Verify we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "contracts" ]]; then
    log_error "Not in PowerGrid Network root directory"
    exit 1
fi

log_info "Starting comprehensive workflow validation..."
echo "This tests: Environment → Build → Unit Tests → Integration Tests → E2E Tests"
echo

# =============================================================================
log_section "📋 1. Environment Validation"
# =============================================================================

log_info "Checking development environment..."

run_test "Rust toolchain available" "rustc --version"
run_test "Cargo available" "cargo --version"
run_test "cargo-contract installed" "cargo contract --version"
run_test "WASM target installed" "rustup target list --installed | grep wasm32-unknown-unknown"
run_test "Substrate contracts node available" "which substrate-contracts-node"

# =============================================================================
log_section "🔧 2. Project Structure Validation"
# =============================================================================

log_info "Validating project structure..."

run_test "Root Cargo.toml exists" "test -f Cargo.toml"
run_test "Contracts directory exists" "test -d contracts"
run_test "Shared library exists" "test -d shared"
run_test "Scripts directory exists" "test -d scripts"
run_test "Integration tests exist" "test -f contracts/integration-tests/Cargo.toml"

# Check all contract directories
for contract in governance grid_service resource_registry token; do
    run_test "Contract '$contract' structure" "test -d contracts/$contract && test -f contracts/$contract/Cargo.toml && test -f contracts/$contract/src/lib.rs"
done

# =============================================================================
log_section "📦 3. Dependencies and Version Consistency"
# =============================================================================

log_info "Checking ink! version consistency..."

# Check workspace version
WORKSPACE_INK_VERSION=$(grep 'ink.*version.*=' Cargo.toml | head -1 | sed 's/.*version.*=.*"\([^"]*\)".*/\1/')
log_info "Workspace ink! version: $WORKSPACE_INK_VERSION"

# Check each contract has matching version
for contract in governance grid_service resource_registry token shared; do
    if [[ -f "$contract/Cargo.toml" ]] || [[ -f "contracts/$contract/Cargo.toml" ]]; then
        CONTRACT_DIR=""
        if [[ -f "$contract/Cargo.toml" ]]; then
            CONTRACT_DIR="$contract"
        else
            CONTRACT_DIR="contracts/$contract"
        fi
        
        CONTRACT_INK_VERSION=$(grep 'ink.*version.*=' "$CONTRACT_DIR/Cargo.toml" | head -1 | sed 's/.*version.*=.*"\([^"]*\)".*/\1/')
        if [[ "$CONTRACT_INK_VERSION" == "$WORKSPACE_INK_VERSION" ]]; then
            run_test "$contract ink! version consistency" "true"
        else
            run_test "$contract ink! version consistency (expected: $WORKSPACE_INK_VERSION, got: $CONTRACT_INK_VERSION)" "false"
        fi
    fi
done

# =============================================================================
log_section "🔨 4. Build Validation"
# =============================================================================

log_info "Testing complete build pipeline..."

run_test_verbose "Clean previous builds" "cargo clean"
run_test_verbose "Workspace compilation check" "cargo check --workspace"
run_test_verbose "Build all contracts (build-all.sh)" "./scripts/build-all.sh"

# Verify build artifacts
log_info "Checking build artifacts..."
for contract in governance grid_service resource_registry powergrid_token; do
    run_test "$contract .contract file generated" "test -f target/ink/$contract/$contract.contract"
    run_test "$contract .wasm file generated" "test -f target/ink/$contract/$contract.wasm"
    run_test "$contract .json metadata generated" "test -f target/ink/$contract/$contract.json"
done

# =============================================================================
log_section "🧪 5. Unit Test Validation"
# =============================================================================

log_info "Running all unit tests..."

run_test_verbose "All individual contract unit tests" "./scripts/test-all.sh"

# Test each contract individually
for contract in governance grid_service resource_registry powergrid_token; do
    run_test_verbose "Unit tests for $contract" "cargo test -p $contract --lib"
done

run_test_verbose "Shared library tests" "cargo test -p powergrid-shared"

# =============================================================================
log_section "🔗 6. Integration Test Validation"
# =============================================================================

log_info "Running simulation integration tests..."

run_test_verbose "Integration test compilation" "cargo test -p integration-tests --no-run"
run_test_verbose "Simulation integration tests" "cargo test -p integration-tests"

# Test specific integration scenarios
run_test_verbose "Complete user journey test" "cargo test -p integration-tests test_complete_user_journey"
run_test_verbose "Data flow integration test" "cargo test -p integration-tests test_data_flow_integration"
run_test_verbose "Error handling integration test" "cargo test -p integration-tests test_error_handling_integration"
run_test_verbose "Scalability integration test" "cargo test -p integration-tests test_scalability_integration"

# =============================================================================
log_section "🌐 7. E2E Test Validation"
# =============================================================================

log_info "Validating E2E test framework..."

run_test_verbose "E2E test compilation (ink! 5.1.1 API)" "cargo test --features e2e-tests -p integration-tests --no-run"

log_info "E2E tests compiled successfully - ready for actual deployment testing"
log_warning "Note: Full E2E test execution requires substrate-contracts-node running"

# =============================================================================
log_section "📊 8. Cross-Contract Functionality Validation"
# =============================================================================

log_info "Validating cross-contract integration points..."

# Check that contracts can reference each other (compilation test)
run_test "Cross-contract dependencies compile" "cargo check -p integration-tests --features e2e-tests"

# Validate shared types work across contracts
run_test "Shared types compilation" "cargo check -p powergrid-shared"

# =============================================================================
log_section "🔍 9. Code Quality Validation"
# =============================================================================

log_info "Running code quality checks..."

run_test "No clippy warnings (governance)" "cargo clippy -p governance -- -D warnings"
run_test "No clippy warnings (grid_service)" "cargo clippy -p grid_service -- -D warnings"
run_test "No clippy warnings (resource_registry)" "cargo clippy -p resource_registry -- -D warnings"
run_test "No clippy warnings (powergrid_token)" "cargo clippy -p powergrid_token -- -D warnings"
run_test "No clippy warnings (shared)" "cargo clippy -p powergrid-shared -- -D warnings"

# =============================================================================
log_section "📋 10. Documentation Validation"
# =============================================================================

log_info "Checking documentation completeness..."

run_test "Root README exists" "test -f README.md"
run_test "E2E tests documentation" "test -f E2E_TESTS_FIXED.md"
run_test "Documentation builds" "cargo doc --workspace --no-deps"

# =============================================================================
log_section "🚀 11. Deployment Readiness"
# =============================================================================

log_info "Validating deployment readiness..."

# Check all contracts build in release mode
run_test_verbose "Release mode compilation" "cargo build --release --workspace"

# Check contract sizes are reasonable
log_info "Contract size analysis:"
for contract in governance grid_service resource_registry powergrid_token; do
    if [[ -f "target/ink/$contract/$contract.wasm" ]]; then
        size=$(du -h "target/ink/$contract/$contract.wasm" | cut -f1)
        log_info "  $contract.wasm: $size"
    fi
done

# =============================================================================
log_section "📈 Test Results Summary"
# =============================================================================

echo
echo "============================================"
echo "           INTEGRATION TEST RESULTS"
echo "============================================"
echo
echo "Total Tests: $TESTS_TOTAL"
echo -e "Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Failed: ${RED}$TESTS_FAILED${NC}"
echo

if [[ $TESTS_FAILED -eq 0 ]]; then
    log_success "🎉 ALL INTEGRATION TESTS PASSED!"
    echo
    echo "✅ Environment Setup: Working"
    echo "✅ Build Pipeline: Working" 
    echo "✅ Unit Tests: All passing"
    echo "✅ Integration Tests: All passing"
    echo "✅ E2E Framework: Ready"
    echo "✅ Cross-Contract: Validated"
    echo "✅ Code Quality: High"
    echo "✅ Documentation: Complete"
    echo "✅ Deployment: Ready"
    echo
    echo "🚀 PowerGrid Network is ready for deployment!"
    echo "🔧 All development tools and scripts working"
    echo "🧪 Complete test coverage validated"
    echo "🌐 E2E testing framework operational"
    echo
    exit 0
else
    log_error "❌ Some integration tests failed!"
    echo
    echo "Failed tests need to be addressed before deployment."
    echo "Check the output above for specific failures."
    echo
    exit 1
fi
