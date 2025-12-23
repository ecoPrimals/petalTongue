#!/usr/bin/env bash
#
# Check Prerequisites
#
# Verify that BiomeOS, petalTongue, and primals are available

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PETALTONGUE_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BIOMEOS_DIR="$(cd "$PETALTONGUE_DIR/../biomeOS" && pwd)"

echo "Checking prerequisites..."
echo ""

# Check BiomeOS binary
if [ -f "$BIOMEOS_DIR/target/release/biomeos" ]; then
    echo -e "${GREEN}✅ BiomeOS binary exists${NC}"
else
    echo -e "${RED}❌ BiomeOS binary not found${NC}"
    echo -e "${YELLOW}   Run: cd $BIOMEOS_DIR && cargo build --release${NC}"
    exit 1
fi

# Check petalTongue binary
if [ -f "$PETALTONGUE_DIR/target/release/petal-tongue-ui" ]; then
    echo -e "${GREEN}✅ petalTongue binary exists${NC}"
else
    echo -e "${RED}❌ petalTongue binary not found${NC}"
    echo -e "${YELLOW}   Run: cd $PETALTONGUE_DIR && cargo build --release${NC}"
    exit 1
fi

# Check for primals in bin/
PRIMAL_COUNT=$(find "$BIOMEOS_DIR/bin/primals/" -maxdepth 1 -type f -executable 2>/dev/null | wc -l)
if [ "$PRIMAL_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅ Primals found in bin/: $PRIMAL_COUNT binaries${NC}"
    echo -e "   ${YELLOW}(beardog, nestgate, songbird-*, toadstool-*, etc.)${NC}"
else
    echo -e "${YELLOW}⚠️  No primals found in bin/primals/${NC}"
    echo -e "   ${YELLOW}This is okay - demos will work with mock data${NC}"
    echo -e "   ${YELLOW}To use real primals, build them in parent directories${NC}"
fi

# Check port 3000 availability
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Port 3000 is already in use${NC}"
    echo -e "   ${YELLOW}BiomeOS may already be running, or another service is using it${NC}"
else
    echo -e "${GREEN}✅ Port 3000 is available${NC}"
fi

echo ""
echo -e "${GREEN}✅ All prerequisites met!${NC}"
echo ""

