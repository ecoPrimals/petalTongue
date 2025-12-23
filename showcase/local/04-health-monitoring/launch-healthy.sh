#!/usr/bin/env bash
#
# Launch Healthy Ecosystem
#
# Launch primals in healthy state

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="$(cd "$SCRIPT_DIR/../../../biomeOS" && pwd)"

mkdir -p "$SCRIPT_DIR/logs"

primals=("beardog" "nestgate")

echo -e "${CYAN}🚀 Launching Healthy Ecosystem${NC}"

launched=0
for primal in "${primals[@]}"; do
    PRIMAL_BIN="$BIOMEOS_DIR/bin/primals/$primal"
    
    if [ ! -f "$PRIMAL_BIN" ]; then
        echo -e "${YELLOW}⚠️  Skipping $primal (binary not found)${NC}"
        continue
    fi
    
    if pgrep -f "$primal" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  $primal already running${NC}"
        ((launched++))
        continue
    fi
    
    echo -e "${GREEN}✅ Launching $primal (healthy)...${NC}"
    "$PRIMAL_BIN" > "$SCRIPT_DIR/logs/$primal.log" 2>&1 &
    PID=$!
    echo $PID > "$SCRIPT_DIR/logs/$primal.pid"
    ((launched++))
    
    sleep 1
done

echo -e "${GREEN}✅ Launched $launched primals in healthy state${NC}"

