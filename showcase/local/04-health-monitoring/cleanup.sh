#!/usr/bin/env bash
#
# Cleanup
#
# Stop all primals

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${CYAN}🧹 Cleaning up primals...${NC}"

stopped=0

if [ -d "$SCRIPT_DIR/logs" ]; then
    for pidfile in "$SCRIPT_DIR/logs/"*.pid 2>/dev/null; do
        if [ -f "$pidfile" ]; then
            PID=$(cat "$pidfile")
            PRIMAL_NAME=$(basename "$pidfile" .pid)
            
            if ps -p $PID > /dev/null 2>&1; then
                echo -e "${YELLOW}⏸️  Stopping $PRIMAL_NAME (PID: $PID)${NC}"
                kill $PID 2>/dev/null || true
                ((stopped++))
                sleep 0.5
            fi
            
            rm "$pidfile"
        fi
    done
fi

# Backup cleanup
for primal in beardog nestgate songbird toadstool squirrel; do
    if pgrep -f "$primal" > /dev/null 2>&1; then
        pkill -f "$primal" 2>/dev/null || true
    fi
done

echo -e "${GREEN}✅ Cleanup complete ($stopped primals stopped)${NC}"

