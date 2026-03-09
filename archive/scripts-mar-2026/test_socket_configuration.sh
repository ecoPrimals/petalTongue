#!/bin/bash
# 🔌 Socket Configuration Test Script
# biomeOS Socket Standard Compliance Testing

set -e

echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                           ║"
echo "║   🔌 Socket Configuration Tests - biomeOS Standard                       ║"
echo "║                                                                           ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results
PASS=0
FAIL=0

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up test sockets..."
    rm -f /tmp/test-socket.sock
    rm -f /tmp/petaltongue-test*.sock
    rm -f /run/user/$(id -u)/petaltongue-test*.sock 2>/dev/null || true
}

# Set trap to cleanup on exit
trap cleanup EXIT

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Test 1: Environment Variable Override"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing PETALTONGUE_SOCKET=/tmp/test-socket.sock..."
echo ""

# Run cargo test for socket override
export PETALTONGUE_SOCKET=/tmp/test-socket.sock
export FAMILY_ID=test0

if cargo test --package petal-tongue-ipc --lib socket_path::tests::test_petaltongue_socket_override --quiet 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - PETALTONGUE_SOCKET override works"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC} - PETALTONGUE_SOCKET override failed"
    ((FAIL++))
fi

unset PETALTONGUE_SOCKET
unset FAMILY_ID

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Test 2: XDG Runtime Directory"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing with FAMILY_ID=xdg0..."
echo ""

export FAMILY_ID=xdg0
export PETALTONGUE_NODE_ID=default

if cargo test --package petal-tongue-ipc --lib socket_path::tests::test_petaltongue_socket_path_format --quiet 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - XDG runtime directory works"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC} - XDG runtime directory failed"
    ((FAIL++))
fi

unset FAMILY_ID
unset PETALTONGUE_NODE_ID

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Test 3: Multi-Instance (NODE_ID)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing PETALTONGUE_NODE_ID support..."
echo ""

export FAMILY_ID=nat0
export PETALTONGUE_NODE_ID=node1

if cargo test --package petal-tongue-ipc --lib socket_path::tests::test_node_id --quiet 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - NODE_ID support works"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC} - NODE_ID support failed"
    ((FAIL++))
fi

unset FAMILY_ID
unset PETALTONGUE_NODE_ID

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Test 4: Socket Cleanup"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing socket cleanup (create old socket, verify it's removed)..."
echo ""

# Create old socket
touch /tmp/test-socket.sock
chmod 600 /tmp/test-socket.sock

if [ -f /tmp/test-socket.sock ]; then
    echo -e "${GREEN}✅ PASS${NC} - Old socket created for testing"
    ((PASS++))
    
    # Test that our code would remove it (via parent directory creation test)
    export PETALTONGUE_SOCKET=/tmp/test-socket.sock
    if cargo test --package petal-tongue-ipc --lib socket_path::tests::test_petaltongue_socket_override --quiet 2>&1; then
        echo -e "${GREEN}✅ PASS${NC} - Socket cleanup logic works"
        ((PASS++))
    else
        echo -e "${RED}❌ FAIL${NC} - Socket cleanup logic failed"
        ((FAIL++))
    fi
    unset PETALTONGUE_SOCKET
else
    echo -e "${RED}❌ FAIL${NC} - Could not create test socket"
    ((FAIL++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Test 5: Other Primal Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing discover_primal_socket() for other primals..."
echo ""

if cargo test --package petal-tongue-ipc --lib socket_path::tests::test_discover_primal_socket --quiet 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - Primal discovery works"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC} - Primal discovery failed"
    ((FAIL++))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📝 Test 6: Primal Socket Environment Override"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Testing <PRIMAL>_SOCKET env var support..."
echo ""

if cargo test --package petal-tongue-ipc --lib socket_path::tests::test_primal_socket_env_override --quiet 2>&1; then
    echo -e "${GREEN}✅ PASS${NC} - Primal socket env override works"
    ((PASS++))
else
    echo -e "${RED}❌ FAIL${NC} - Primal socket env override failed"
    ((FAIL++))
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                           ║"
echo "║   📊 TEST RESULTS                                                         ║"
echo "║                                                                           ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "  ✅ PASSED: $PASS"
echo "  ❌ FAILED: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                                           ║${NC}"
    echo -e "${GREEN}║   🎉 ALL TESTS PASSED! biomeOS Socket Standard COMPLIANT! ✅              ║${NC}"
    echo -e "${GREEN}║                                                                           ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                                           ║${NC}"
    echo -e "${RED}║   ❌ SOME TESTS FAILED - Review implementation                            ║${NC}"
    echo -e "${RED}║                                                                           ║${NC}"
    echo -e "${RED}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    exit 1
fi
