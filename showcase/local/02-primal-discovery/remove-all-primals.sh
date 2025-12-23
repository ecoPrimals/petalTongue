#!/usr/bin/env bash
#
# Remove All Primals
#
# Stop all running primals

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Stop primals by PID files
for pidfile in "$SCRIPT_DIR/logs/"*.pid; do
    if [ -f "$pidfile" ]; then
        PID=$(cat "$pidfile")
        PRIMAL_NAME=$(basename "$pidfile" .pid)
        
        if ps -p $PID > /dev/null 2>&1; then
            echo -e "${YELLOW}⏸️  Stopping $PRIMAL_NAME (PID: $PID)${NC}"
            kill $PID 2>/dev/null || true
            sleep 0.5
        fi
        
        rm "$pidfile"
    fi
done

# Also kill by process name (in case PID files are missing)
for primal in beardog nestgate songbird toadstool squirrel; do
    if pgrep -f "$primal" > /dev/null 2>&1; then
        pkill -f "$primal" 2>/dev/null || true
    fi
done

echo -e "${GREEN}✅ All primals stopped${NC}"

