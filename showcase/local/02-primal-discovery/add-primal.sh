#!/usr/bin/env bash
#
# Add Primal
#
# Launch a primal and watch it be discovered

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Primal name required${NC}"
    echo "Usage: ./add-primal.sh <primal-name>"
    echo "Example: ./add-primal.sh beardog"
    exit 1
fi

PRIMAL="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="$(cd "$SCRIPT_DIR/../../../biomeOS" && pwd)"

# Create logs directory
mkdir -p "$SCRIPT_DIR/logs"

# Check if binary exists
PRIMAL_BIN="$BIOMEOS_DIR/bin/primals/$PRIMAL"

if [ ! -f "$PRIMAL_BIN" ]; then
    echo -e "${RED}❌ Primal binary not found: $PRIMAL_BIN${NC}"
    exit 1
fi

# Check if already running
if pgrep -f "$PRIMAL" > /dev/null; then
    echo -e "${YELLOW}⚠️  $PRIMAL already running${NC}"
    exit 0
fi

echo -e "${YELLOW}🚀 Launching $PRIMAL...${NC}"

# Launch primal
"$PRIMAL_BIN" > "$SCRIPT_DIR/logs/$PRIMAL.log" 2>&1 &
PRIMAL_PID=$!

# Save PID
echo $PRIMAL_PID > "$SCRIPT_DIR/logs/$PRIMAL.pid"

echo -e "${GREEN}✅ $PRIMAL started (PID: $PRIMAL_PID)${NC}"
echo -e "   Logs: $SCRIPT_DIR/logs/$PRIMAL.log"

