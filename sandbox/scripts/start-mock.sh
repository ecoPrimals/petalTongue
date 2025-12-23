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
    echo -e "${YELLOW}⚠️  Mock server not found. Building...${NC}"
    cd "$SANDBOX_DIR"
    
    # Create simple mock server if it doesn't exist
    if [ ! -f "$MOCK_DIR/Cargo.toml" ]; then
        echo "Mock server implementation pending."
        echo "For now, use built-in mock mode in BiomeOSClient."
        echo ""
        echo "Run petalTongue with:"
        echo "  cargo run --release -p petal-tongue-ui"
        echo ""
        echo "(It will automatically use mock data when BiomeOS is unavailable)"
        exit 0
    fi
fi

# Start the mock server
cd "$MOCK_DIR"
echo -e "${GREEN}✓${NC} Starting mock server on ${BLUE}http://localhost:3333${NC}"
echo ""
echo "Endpoints:"
echo "  GET /api/v1/primals   - Discover primals"
echo "  GET /api/v1/topology  - Get topology edges"
echo "  GET /api/v1/health    - Ecosystem health"
echo ""
echo "Press Ctrl+C to stop"
echo ""

cargo run --release

