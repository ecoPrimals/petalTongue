#!/usr/bin/env bash
#
# Cleanup
#
# Stop BiomeOS and petalTongue, clean up logs

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Cleaning up..."
echo ""

# Stop petalTongue
if [ -f "$SCRIPT_DIR/logs/petaltongue.pid" ]; then
    PETALTONGUE_PID=$(cat "$SCRIPT_DIR/logs/petaltongue.pid")
    if ps -p $PETALTONGUE_PID > /dev/null 2>&1; then
        echo -e "${YELLOW}⏸️  Stopping petalTongue (PID: $PETALTONGUE_PID)${NC}"
        kill $PETALTONGUE_PID
        sleep 1
        echo -e "${GREEN}✅ petalTongue stopped${NC}"
    else
        echo -e "${YELLOW}⚠️  petalTongue not running${NC}"
    fi
    rm "$SCRIPT_DIR/logs/petaltongue.pid"
fi

# Stop BiomeOS
if [ -f "$SCRIPT_DIR/logs/biomeos.pid" ]; then
    BIOMEOS_PID=$(cat "$SCRIPT_DIR/logs/biomeos.pid")
    if ps -p $BIOMEOS_PID > /dev/null 2>&1; then
        echo -e "${YELLOW}⏸️  Stopping BiomeOS (PID: $BIOMEOS_PID)${NC}"
        kill $BIOMEOS_PID
        sleep 1
        echo -e "${GREEN}✅ BiomeOS stopped${NC}"
    else
        echo -e "${YELLOW}⚠️  BiomeOS not running${NC}"
    fi
    rm "$SCRIPT_DIR/logs/biomeos.pid"
fi

# Clean up logs (optional - comment out to keep logs)
# rm -f "$SCRIPT_DIR/logs/"*.log

echo ""
echo -e "${GREEN}✅ Cleanup complete!${NC}"
echo ""

