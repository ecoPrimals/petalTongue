#!/usr/bin/env bash
#
# Launch BearDog (Security Primal) Only
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

# Check if beardog binary exists
BEARDOG_BIN="$BIOMEOS_DIR/bin/primals/beardog"

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}❌ BearDog binary not found: $BEARDOG_BIN${NC}"
    echo -e "${YELLOW}   Build beardog in the parent directory first${NC}"
    exit 1
fi

# Check if already running
if pgrep -f "beardog" > /dev/null; then
    echo -e "${YELLOW}⚠️  BearDog already running${NC}"
    echo -e "${GREEN}✅ Skipping launch${NC}"
    exit 0
fi

echo "Launching BearDog (Security Primal)..."

# Launch beardog
"$BEARDOG_BIN" > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
BEARDOG_PID=$!

# Save PID
echo $BEARDOG_PID > "$SCRIPT_DIR/logs/beardog.pid"

echo -e "${GREEN}✅ BearDog started (PID: $BEARDOG_PID)${NC}"
echo -e "   Type: Security"
echo -e "   Expected Audio: Deep Bass"
echo -e "   Logs: $SCRIPT_DIR/logs/beardog.log"
echo ""
echo -e "${YELLOW}⏳ Waiting for BiomeOS to discover it...${NC}"

# Wait a bit for discovery
sleep 3

echo -e "${GREEN}✅ BearDog should be visible in petalTongue now!${NC}"
echo -e "${YELLOW}   (If not, wait 5s for auto-refresh or click 'Refresh Now')${NC}"
echo ""

