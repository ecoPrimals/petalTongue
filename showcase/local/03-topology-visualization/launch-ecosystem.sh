#!/usr/bin/env bash
#
# Launch Ecosystem
#
# Launch a full set of primals to create a realistic ecosystem

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="$(cd "$SCRIPT_DIR/../../../biomeOS" && pwd)"

# Create logs directory
mkdir -p "$SCRIPT_DIR/logs"

# Primals to launch
primals=("beardog" "nestgate")  # Start with basics that we know exist

echo -e "${CYAN}🚀 Launching Full Ecosystem${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

launched=0
skipped=0

for primal in "${primals[@]}"; do
    PRIMAL_BIN="$BIOMEOS_DIR/bin/primals/$primal"
    
    if [ ! -f "$PRIMAL_BIN" ]; then
        echo -e "${YELLOW}⚠️  Skipping $primal (binary not found)${NC}"
        ((skipped++))
        continue
    fi
    
    if pgrep -f "$primal" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  $primal already running${NC}"
        ((launched++))
        continue
    fi
    
    echo -e "${GREEN}✅ Launching $primal...${NC}"
    "$PRIMAL_BIN" > "$SCRIPT_DIR/logs/$primal.log" 2>&1 &
    PID=$!
    echo $PID > "$SCRIPT_DIR/logs/$primal.pid"
    ((launched++))
    
    # Small delay between launches
    sleep 1
done

echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Ecosystem Launch Complete${NC}"
echo -e "   ${GREEN}Launched:${NC} $launched primals"
if [ $skipped -gt 0 ]; then
    echo -e "   ${YELLOW}Skipped:${NC} $skipped primals (binaries not found)"
fi
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${YELLOW}📝 Primal logs available in:${NC} $SCRIPT_DIR/logs/"

