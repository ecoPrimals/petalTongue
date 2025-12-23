#!/usr/bin/env bash
#
# Run a showcase demo
#
# Usage: ./run-demo.sh <demo-number>
#
# Examples:
#   ./run-demo.sh 01   # Basic topology
#   ./run-demo.sh 04   # Audio-only (accessibility)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PINK='\033[1;35m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_DIR="$(dirname "$SCRIPT_DIR")"
PETALTONGUE_DIR="$(dirname "$SHOWCASE_DIR")"

# Parse arguments
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Demo number required${NC}"
    echo ""
    echo "Usage: ./run-demo.sh <demo-number>"
    echo ""
    echo "Available demos:"
    echo "  01 - Basic Topology (2-3 min)"
    echo "  02 - Degraded System (3-5 min)"
    echo "  03 - Scaling Event (5-7 min)"
    echo "  04 - Audio-Only Experience (5-10 min)"
    echo "  05 - Production Scale (5-7 min)"
    exit 1
fi

DEMO_NUM="$1"
DEMO_DIR="$SHOWCASE_DIR/demos/${DEMO_NUM}"

# Validate demo exists
if [ ! -d "$DEMO_DIR" ]; then
    echo -e "${RED}Error: Demo $DEMO_NUM not found${NC}"
    echo ""
    echo "Available demos:"
    ls -1 "$SHOWCASE_DIR/demos/" 2>/dev/null || echo "  (none yet)"
    exit 1
fi

# Check if setup script exists
if [ -f "$DEMO_DIR/setup.sh" ]; then
    echo -e "${BLUE}📦 Running demo setup...${NC}"
    cd "$DEMO_DIR"
    bash setup.sh
else
    echo -e "${YELLOW}⚠️  No setup script found for demo $DEMO_NUM${NC}"
    echo "Running with current configuration..."
fi

# Display presenter notes if available
if [ -f "$DEMO_DIR/script.md" ]; then
    echo ""
    echo -e "${PINK}🎤 Presenter Notes:${NC}"
    echo -e "${PINK}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    head -20 "$DEMO_DIR/script.md"
    echo ""
    echo -e "${BLUE}(See full script at: demos/$DEMO_NUM/script.md)${NC}"
fi

echo ""
echo -e "${GREEN}✓${NC} Demo $DEMO_NUM ready!"
echo ""
echo "Press Enter to continue, or Ctrl+C to cancel..."
read -r

# Demo is now ready - petalTongue should already be running from setup.sh
echo -e "${GREEN}🎬 Demo $DEMO_NUM is live!${NC}"

