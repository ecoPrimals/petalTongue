#!/usr/bin/env bash
#
# Remove Primal
#
# Stop a primal and watch it disappear from discovery

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Primal name required${NC}"
    echo "Usage: ./remove-primal.sh <primal-name>"
    echo "Example: ./remove-primal.sh beardog"
    exit 1
fi

PRIMAL="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${YELLOW}⏸️  Stopping $PRIMAL...${NC}"

# Stop by PID file if exists
if [ -f "$SCRIPT_DIR/logs/$PRIMAL.pid" ]; then
    PID=$(cat "$SCRIPT_DIR/logs/$PRIMAL.pid")
    if ps -p $PID > /dev/null 2>&1; then
        kill $PID 2>/dev/null || true
        sleep 1
    fi
    rm "$SCRIPT_DIR/logs/$PRIMAL.pid"
fi

# Also try by process name
if pgrep -f "$PRIMAL" > /dev/null 2>&1; then
    pkill -f "$PRIMAL" 2>/dev/null || true
fi

echo -e "${GREEN}✅ $PRIMAL stopped${NC}"

