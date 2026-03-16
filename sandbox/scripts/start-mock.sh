#!/usr/bin/env bash
# Start mock BiomeOS server for petalTongue development

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SANDBOX_DIR="$(dirname "$SCRIPT_DIR")"
MOCK_DIR="$SANDBOX_DIR/mock-biomeos"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 Starting petalTongue Sandbox Mock Server${NC}"
echo ""

# Check if mock server exists
if [ ! -d "$MOCK_DIR" ]; then
    echo -e "${YELLOW}⚠️  Mock server directory not found at $MOCK_DIR${NC}"
    echo "Expected: $MOCK_DIR"
    exit 1
fi

# Check if mock server is built
if [ ! -f "$MOCK_DIR/target/release/mock-biomeos" ]; then
    echo -e "${YELLOW}⚠️  Mock server not built. Building now...${NC}"
    cd "$MOCK_DIR"
    cargo build --release
    echo ""
fi

# Start the mock server
cd "$MOCK_DIR"
echo -e "${GREEN}✓${NC} Starting mock server on ${BLUE}http://localhost:3000${NC}"
echo ""
echo "Endpoints:"
echo "  GET http://localhost:3000/                 - Server info"
echo "  GET http://localhost:3000/api/v1/primals   - Discover primals"
echo "  GET http://localhost:3000/api/v1/topology  - Get topology edges"
echo "  GET http://localhost:3000/api/v1/health    - Ecosystem health"
echo ""
echo "Scenarios: $SANDBOX_DIR/scenarios/"
echo "Hot-reload: Edit scenarios/*.json and server auto-reloads"
echo ""
echo "Test with petalTongue:"
echo "  BIOMEOS_URL=http://localhost:3000 cargo run --release -- ui"
echo ""
echo "Press Ctrl+C to stop"
echo ""

./target/release/mock-biomeos

