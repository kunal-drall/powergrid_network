#!/usr/bin/env bash
set -e

# PowerGrid Network - Comprehensive Quality Review Script
# This script performs a complete milestone review as requested

echo "🔍 PowerGrid Network - Comprehensive Quality Review"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track results
PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

log_pass() {
    echo -e "${GREEN}✅ PASS: $1${NC}"
    ((PASS_COUNT++))
}

log_fail() {
    echo -e "${RED}❌ FAIL: $1${NC}"
    ((FAIL_COUNT++))
}

log_warn() {
    echo -e "${YELLOW}⚠️  WARN: $1${NC}"
    ((WARN_COUNT++))
}

log_info() {
    echo -e "${BLUE}ℹ️  INFO: $1${NC}"
}

echo -e "${BLUE}Phase 1: Infrastructure & Tooling Verification${NC}"
echo "==============================================="

# Check Rust version
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    log_pass "Rust compiler available: $RUST_VERSION"
else
    log_fail "Rust compiler not found"
fi

# Check cargo-contract version (if available)
if command -v cargo-contract &> /dev/null; then
    CONTRACT_VERSION=$(cargo-contract --version)
    if [[ $CONTRACT_VERSION == *"5.0.3"* ]] || [[ $CONTRACT_VERSION == *"5.0.4"* ]] || [[ $CONTRACT_VERSION == *"5.0.5"* ]]; then
        log_pass "cargo-contract version meets v5.0.3+ requirement: $CONTRACT_VERSION"
    else
        log_warn "cargo-contract version may not meet v5.0.3+ requirement: $CONTRACT_VERSION"
    fi
else
    log_warn "cargo-contract not found (will be installed during setup)"
fi

# Check WASM target
if rustup target list --installed | grep -q wasm32-unknown-unknown; then
    log_pass "WASM target (wasm32-unknown-unknown) is installed"
else
    log_warn "WASM target not installed (will be added during setup)"
fi

echo ""
echo -e "${BLUE}Phase 2: Contract Code Quality Review${NC}"
echo "===================================="

# Check for #[allow] directives (should be minimal)
ALLOW_COUNT=$(find contracts/ -name "*.rs" -exec grep -l "#\[allow" {} \; | wc -l)
if [ "$ALLOW_COUNT" -eq 0 ]; then
    log_pass "No #[allow] directives found in contract code"
elif [ "$ALLOW_COUNT" -le 2 ]; then
    log_warn "$ALLOW_COUNT files contain #[allow] directives (should be justified)"
    find contracts/ -name "*.rs" -exec grep -Hn "#\[allow" {} \;
else
    log_fail "Too many #[allow] directives found: $ALLOW_COUNT files"
fi

# Check for proper error handling patterns
ERROR_PATTERN_COUNT=$(find contracts/ -name "*.rs" -exec grep -l "Result<" {} \; | wc -l)
if [ "$ERROR_PATTERN_COUNT" -ge 4 ]; then
    log_pass "Proper error handling patterns found in contracts"
else
    log_warn "Limited error handling patterns found"
fi

# Check for reentrancy protection
REENTRANCY_COUNT=$(find contracts/ -name "*.rs" -exec grep -l "entered" {} \; | wc -l)
if [ "$REENTRANCY_COUNT" -ge 2 ]; then
    log_pass "Reentrancy protection mechanisms found"
else
    log_warn "Limited reentrancy protection found"
fi

echo ""
echo -e "${BLUE}Phase 3: Testing Infrastructure${NC}"
echo "=============================="

# Check workspace compilation
log_info "Checking workspace compilation..."
if timeout 180 cargo check --workspace &> /dev/null; then
    log_pass "Workspace compiles successfully"
else
    log_fail "Workspace compilation failed or timed out"
fi

# Check unit tests exist
UNIT_TEST_COUNT=$(find contracts/ -name "*.rs" -exec grep -l "#\[test\]" {} \; | wc -l)
if [ "$UNIT_TEST_COUNT" -ge 4 ]; then
    log_pass "Unit tests found in $UNIT_TEST_COUNT contract files"
else
    log_warn "Limited unit test coverage: $UNIT_TEST_COUNT files with tests"
fi

# Check e2e tests exist
if [ -f "contracts/integration-tests/src/real_e2e_tests.rs" ]; then
    log_pass "E2E tests implementation found"
else
    log_fail "E2E tests implementation missing"
fi

# Check e2e tests feature
if grep -q "e2e-tests" contracts/integration-tests/Cargo.toml; then
    log_pass "E2E tests feature properly configured"
else
    log_fail "E2E tests feature not configured"
fi

echo ""
echo -e "${BLUE}Phase 4: Docker Cross-Platform Setup${NC}"
echo "==================================="

# Check Dockerfile exists and is well-formed
if [ -f "Dockerfile" ]; then
    log_pass "Dockerfile exists"
    
    # Check for multi-stage build
    if grep -q "FROM.*AS" Dockerfile; then
        log_pass "Multi-stage Docker build configured"
    else
        log_warn "Single-stage Docker build (consider multi-stage for optimization)"
    fi
    
    # Check for cargo-contract installation
    if grep -q "cargo-contract" Dockerfile; then
        log_pass "cargo-contract installation found in Dockerfile"
    else
        log_fail "cargo-contract installation missing from Dockerfile"
    fi
else
    log_fail "Dockerfile missing"
fi

# Check docker-compose.yml
if [ -f "docker-compose.yml" ]; then
    log_pass "Docker Compose configuration exists"
    
    # Check for health checks
    if grep -q "healthcheck" docker-compose.yml; then
        log_pass "Health checks configured in Docker Compose"
    else
        log_warn "No health checks found in Docker Compose"
    fi
else
    log_fail "docker-compose.yml missing"
fi

echo ""
echo -e "${BLUE}Phase 5: Documentation Quality${NC}"
echo "============================="

# Check README.md
if [ -f "README.md" ]; then
    README_SIZE=$(wc -l < README.md)
    if [ "$README_SIZE" -gt 100 ]; then
        log_pass "Comprehensive README.md ($README_SIZE lines)"
    else
        log_warn "README.md may be too brief ($README_SIZE lines)"
    fi
    
    # Check for essential sections
    if grep -qi "installation\|setup" README.md; then
        log_pass "Installation/setup instructions found in README"
    else
        log_warn "Installation instructions missing from README"
    fi
    
    if grep -qi "testing" README.md; then
        log_pass "Testing instructions found in README"
    else
        log_warn "Testing instructions missing from README"
    fi
else
    log_fail "README.md missing"
fi

# Check TESTING.md
if [ -f "TESTING.md" ]; then
    log_pass "TESTING.md documentation exists"
else
    log_fail "TESTING.md missing"
fi

# Check scripts/README.md
if [ -f "scripts/README.md" ]; then
    log_pass "scripts/README.md documentation exists"
else
    log_fail "scripts/README.md missing"
fi

# Check for version consistency
VERSION_INCONSISTENCIES=$(grep -r "5\.0\.1" . --include="*.md" --include="*.sh" --include="Dockerfile" | wc -l)
if [ "$VERSION_INCONSISTENCIES" -eq 0 ]; then
    log_pass "Version consistency maintained (no v5.0.1 references)"
else
    log_warn "$VERSION_INCONSISTENCIES version inconsistencies found"
    grep -r "5\.0\.1" . --include="*.md" --include="*.sh" --include="Dockerfile" || true
fi

echo ""
echo -e "${BLUE}Phase 6: Repository Cleanliness${NC}"
echo "=============================="

# Check .gitignore
if [ -f ".gitignore" ]; then
    GITIGNORE_SIZE=$(wc -l < .gitignore)
    if [ "$GITIGNORE_SIZE" -gt 10 ]; then
        log_pass "Comprehensive .gitignore ($GITIGNORE_SIZE lines)"
    else
        log_warn ".gitignore may be incomplete ($GITIGNORE_SIZE lines)"
    fi
else
    log_fail ".gitignore missing"
fi

# Check for common build artifacts that shouldn't be committed
if find . -name "target" -type d | grep -q .; then
    log_warn "Build artifacts (target directories) found - should be in .gitignore"
else
    log_pass "No build artifacts found in repository"
fi

# Check for large files
LARGE_FILES=$(find . -type f -size +1M 2>/dev/null | grep -v ".git" | wc -l)
if [ "$LARGE_FILES" -eq 0 ]; then
    log_pass "No large files (>1MB) found"
else
    log_warn "$LARGE_FILES large files found (may need LFS or exclusion)"
fi

echo ""
echo -e "${BLUE}Phase 7: Script Quality${NC}"
echo "====================="

# Check script permissions
EXECUTABLE_SCRIPTS=$(find scripts/ -name "*.sh" -executable | wc -l)
TOTAL_SCRIPTS=$(find scripts/ -name "*.sh" | wc -l)

if [ "$EXECUTABLE_SCRIPTS" -eq "$TOTAL_SCRIPTS" ]; then
    log_pass "All scripts ($TOTAL_SCRIPTS) are executable"
else
    log_warn "Some scripts are not executable ($EXECUTABLE_SCRIPTS/$TOTAL_SCRIPTS)"
fi

# Check for proper shebang
SCRIPTS_WITH_SHEBANG=$(find scripts/ -name "*.sh" -exec head -1 {} \; | grep -c "#!/")
if [ "$SCRIPTS_WITH_SHEBANG" -eq "$TOTAL_SCRIPTS" ]; then
    log_pass "All scripts have proper shebang"
else
    log_warn "Some scripts missing shebang ($SCRIPTS_WITH_SHEBANG/$TOTAL_SCRIPTS)"
fi

# Check for set -e (exit on error)
SCRIPTS_WITH_SET_E=$(find scripts/ -name "*.sh" -exec grep -l "set -e" {} \; | wc -l)
if [ "$SCRIPTS_WITH_SET_E" -ge 1 ]; then
    log_pass "Error handling (set -e) found in scripts"
else
    log_warn "Limited error handling in scripts"
fi

echo ""
echo "=================================================="
echo -e "${BLUE}📊 Quality Review Summary${NC}"
echo "=================================================="
echo -e "✅ ${GREEN}PASSED: $PASS_COUNT checks${NC}"
echo -e "⚠️  ${YELLOW}WARNINGS: $WARN_COUNT issues${NC}"
echo -e "❌ ${RED}FAILED: $FAIL_COUNT issues${NC}"

echo ""
if [ "$FAIL_COUNT" -eq 0 ]; then
    if [ "$WARN_COUNT" -eq 0 ]; then
        echo -e "${GREEN}🎉 MILESTONE REVIEW: EXCELLENT QUALITY${NC}"
        echo "✅ All critical requirements met"
        echo "✅ No major issues found"
        echo "✅ Ready for production deployment"
    else
        echo -e "${YELLOW}🎯 MILESTONE REVIEW: GOOD QUALITY${NC}"
        echo "✅ All critical requirements met"
        echo "⚠️  Some minor improvements recommended"
        echo "✅ Ready for production with minor cleanup"
    fi
else
    echo -e "${RED}🔧 MILESTONE REVIEW: NEEDS ATTENTION${NC}"
    echo "❌ Critical issues need to be addressed"
    echo "📋 Please fix the failed checks before production"
fi

echo ""
echo -e "${BLUE}🔍 Recommended Actions:${NC}"
echo "1. Address any FAILED checks immediately"
echo "2. Consider addressing WARNING items for better quality"
echo "3. Run './scripts/setup.sh' to install dependencies"
echo "4. Run './scripts/test-all.sh' to verify unit tests"
echo "5. Test Docker setup with 'docker-compose up'"

exit $FAIL_COUNT