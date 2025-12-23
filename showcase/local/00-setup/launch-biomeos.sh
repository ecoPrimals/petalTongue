#!/usr/bin/env bash
#
# Launch BiomeOS
#
# Start BiomeOS in background for testing

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

# Check if already running
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  BiomeOS already running on port 3000${NC}"
    echo -e "${GREEN}✅ Skipping launch${NC}"
    exit 0
fi

echo "Launching BiomeOS..."

# Create logs directory
mkdir -p "$SCRIPT_DIR/logs"

# Launch BiomeOS in background
cd "$BIOMEOS_DIR"
RUST_LOG=info cargo run --release > "$SCRIPT_DIR/logs/biomeos.log" 2>&1 &
BIOMEOS_PID=$!

# Save PID
echo $BIOMEOS_PID > "$SCRIPT_DIR/logs/biomeos.pid"

echo -e "${GREEN}✅ BiomeOS starting (PID: $BIOMEOS_PID)${NC}"
echo -e "   URL: http://localhost:3000"
echo -e "   Logs: $SCRIPT_DIR/logs/biomeos.log"
echo ""
echo -e "${YELLOW}⏳ Waiting for BiomeOS to be ready...${NC}"

# Wait for BiomeOS to respond (max 30s)
for i in {1..30}; do
    if curl -s http://localhost:3000/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ BiomeOS is ready!${NC}"
        exit 0
    fi
    sleep 1
    echo -n "."
done

echo ""
echo -e "${RED}❌ BiomeOS failed to start within 30 seconds${NC}"
echo -e "${YELLOW}   Check logs: tail -f $SCRIPT_DIR/logs/biomeos.log${NC}"
exit 1

