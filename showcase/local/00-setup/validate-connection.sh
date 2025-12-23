#!/usr/bin/env bash
#
# Validate Connection
#
# Verify petalTongue is connected to BiomeOS

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "Validating petalTongue ↔ BiomeOS connection..."
echo ""

# Check if BiomeOS is accessible
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${GREEN}✅ BiomeOS is accessible${NC}"
else
    echo -e "${RED}❌ BiomeOS is not accessible${NC}"
    exit 1
fi

# Check if petalTongue process is running
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/logs/petaltongue.pid" ]; then
    PETALTONGUE_PID=$(cat "$SCRIPT_DIR/logs/petaltongue.pid")
    if ps -p $PETALTONGUE_PID > /dev/null 2>&1; then
        echo -e "${GREEN}✅ petalTongue process is running${NC}"
    else
        echo -e "${RED}❌ petalTongue process not found${NC}"
        echo -e "${YELLOW}   Check logs: tail -f $SCRIPT_DIR/logs/petaltongue.log${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  petalTongue PID file not found${NC}"
    echo -e "${YELLOW}   This might be okay if launched manually${NC}"
fi

# Check for primals (may be zero)
PRIMAL_RESPONSE=$(curl -s http://localhost:3000/api/v1/primals 2>/dev/null || echo "[]")
PRIMAL_COUNT=$(echo "$PRIMAL_RESPONSE" | grep -o "\"id\"" | wc -l)

if [ "$PRIMAL_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✅ Primals visible: $PRIMAL_COUNT nodes${NC}"
else
    echo -e "${YELLOW}⚠️  No primals running yet${NC}"
    echo -e "   ${YELLOW}This is expected - you can launch primals later${NC}"
fi

echo ""
echo -e "${GREEN}✅ Connection validation complete!${NC}"
echo ""
echo -e "${YELLOW}📝 What to check in petalTongue UI:${NC}"
echo -e "   • Window should be open${NC}"
echo -e "   • Title: '🌸 petalTongue - Universal Representation System'${NC}"
echo -e "   • Graph area (center) - may be empty if no primals${NC}"
echo -e "   • Statistics show: '$PRIMAL_COUNT nodes, 0 edges'${NC}"
echo -e "   • No connection errors${NC}"
echo ""

