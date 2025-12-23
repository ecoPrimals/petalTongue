#!/usr/bin/env bash
#
# Launch petalTongue
#
# Start petalTongue UI pointing at BiomeOS

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PETALTONGUE_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "Launching petalTongue..."
echo ""

# Create logs directory
mkdir -p "$SCRIPT_DIR/logs"

# Launch petalTongue in background
cd "$PETALTONGUE_DIR"
BIOMEOS_URL=http://localhost:3000 \
    cargo run --release -p petal-tongue-ui > "$SCRIPT_DIR/logs/petaltongue.log" 2>&1 &
PETALTONGUE_PID=$!

# Save PID
echo $PETALTONGUE_PID > "$SCRIPT_DIR/logs/petaltongue.pid"

echo -e "${GREEN}✅ petalTongue starting (PID: $PETALTONGUE_PID)${NC}"
echo -e "   Connected to: http://localhost:3000"
echo -e "   Logs: $SCRIPT_DIR/logs/petaltongue.log"
echo ""
echo -e "${YELLOW}⏳ petalTongue UI should open shortly...${NC}"
echo -e "${YELLOW}   (Window may take 3-5 seconds to appear)${NC}"

# Wait a bit for UI to start
sleep 3

echo ""
echo -e "${GREEN}✅ petalTongue launched!${NC}"
echo -e "   ${YELLOW}Check for the petalTongue window${NC}"
echo ""

