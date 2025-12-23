#!/usr/bin/env bash
#
# Launch NestGate (Storage Primal) Only
#

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="$(cd "$SCRIPT_DIR/../../../biomeOS" && pwd)"

# Create logs directory
mkdir -p "$SCRIPT_DIR/logs"

# Check if nestgate binary exists
NESTGATE_BIN="$BIOMEOS_DIR/bin/primals/nestgate"

if [ ! -f "$NESTGATE_BIN" ]; then
    echo -e "${RED}❌ NestGate binary not found: $NESTGATE_BIN${NC}"
    echo -e "${YELLOW}   Build nestgate in the parent directory first${NC}"
    exit 1
fi

# Check if already running
if pgrep -f "nestgate" > /dev/null; then
    echo -e "${YELLOW}⚠️  NestGate already running${NC}"
    echo -e "${GREEN}✅ Skipping launch${NC}"
    exit 0
fi

echo "Launching NestGate (Storage Primal)..."

# Launch nestgate
"$NESTGATE_BIN" > "$SCRIPT_DIR/logs/nestgate.log" 2>&1 &
NESTGATE_PID=$!

# Save PID
echo $NESTGATE_PID > "$SCRIPT_DIR/logs/nestgate.pid"

echo -e "${GREEN}✅ NestGate started (PID: $NESTGATE_PID)${NC}"
echo -e "   Type: Storage"
echo -e "   Expected Audio: Sustained Strings"
echo -e "   Logs: $SCRIPT_DIR/logs/nestgate.log"
echo ""
echo -e "${YELLOW}⏳ Waiting for BiomeOS to discover it...${NC}"

# Wait a bit for discovery
sleep 3

echo -e "${GREEN}✅ NestGate should be visible in petalTongue now!${NC}"
echo -e "${YELLOW}   (If not, wait 5s for auto-refresh or click 'Refresh Now')${NC}"
echo ""

