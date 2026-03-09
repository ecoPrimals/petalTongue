#!/bin/bash
# Systematic Test Fixes for petalTongue
# This script identifies and helps fix test compilation errors

set -e

echo "🔧 petalTongue Test Remediation Script"
echo "======================================"
echo ""

# Colors for output
RED='\033[0.31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to test a single crate
test_crate() {
    local crate=$1
    echo -e "${YELLOW}Testing: $crate${NC}"
    if cargo test --package "$crate" --no-default-features --no-run 2>&1 | tee /tmp/test_output.txt; then
        echo -e "${GREEN}✅ $crate tests compile${NC}"
        return 0
    else
        echo -e "${RED}❌ $crate tests failed${NC}"
        grep "^error" /tmp/test_output.txt | head -10
        return 1
    fi
    echo ""
}

# Test each crate
CRATES=(
    "petal-tongue-core"
    "petal-tongue-entropy"
    "petal-tongue-discovery"
    "petal-tongue-api"
    "petal-tongue-graph"
    "petal-tongue-animation"
    "petal-tongue-ui-core"
    "petal-tongue-modalities"
)

FAILED=()
PASSED=()

for crate in "${CRATES[@]}"; do
    if test_crate "$crate"; then
        PASSED+=("$crate")
    else
        FAILED+=("$crate")
    fi
done

echo ""
echo "======================================"
echo "📊 Test Compilation Summary"
echo "======================================"
echo -e "${GREEN}Passed: ${#PASSED[@]}${NC}"
for crate in "${PASSED[@]}"; do
    echo -e "  ${GREEN}✅${NC} $crate"
done

echo ""
echo -e "${RED}Failed: ${#FAILED[@]}${NC}"
for crate in "${FAILED[@]}"; do
    echo -e "  ${RED}❌${NC} $crate"
done

if [ ${#FAILED[@]} -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 All tests compile successfully!${NC}"
    exit 0
else
    echo ""
    echo -e "${YELLOW}⚠️  Some tests still need fixes${NC}"
    exit 1
fi

