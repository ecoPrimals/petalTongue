#!/usr/bin/env bash
# benchTop Demonstration - The ecoPrimals Desktop Environment
# Smooth. Beautiful. Powerful. Adaptive.

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ASCII Art Banner
cat << 'EOF'
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║         🌸 benchTop - The ecoPrimals Desktop Environment         ║
║                                                                   ║
║         Smooth. Beautiful. Powerful. Adaptive.                    ║
║                                                                   ║
╚═══════════════════════════════════════════════════════════════════╝
EOF

echo ""
echo -e "${BLUE}Welcome to benchTop!${NC}"
echo ""
echo "This demonstration showcases petalTongue as a modern desktop environment."
echo "Think popOS cosmic meets steamOS meets Discord, with Rust ownership."
echo ""

# Menu
echo -e "${GREEN}Select a demonstration:${NC}"
echo ""
echo "  1. Live Ecosystem - Real-time primal coordination"
echo "  2. Graph Builder Studio - Visual graph construction"
echo "  3. RootPulse - Temporal coordination visualization"
echo "  4. Neural API Learning - Adaptive optimization"
echo "  5. Full Tour - Experience all features"
echo "  6. Exit"
echo ""

read -rp "Enter choice [1-6]: " choice

case $choice in
  1)
    echo -e "${BLUE}Launching Live Ecosystem...${NC}"
    ../target/release/petaltongue ui \
      --scenario scenarios/live-ecosystem.json \
      --mode live
    ;;
  2)
    echo -e "${BLUE}Launching Graph Builder Studio...${NC}"
    ../target/release/petaltongue ui \
      --scenario scenarios/graph-studio.json \
      --mode graph-builder
    ;;
  3)
    echo -e "${BLUE}Launching RootPulse Visualization...${NC}"
    ../target/release/petaltongue ui \
      --scenario scenarios/rootpulse-demo.json \
      --mode rootpulse
    ;;
  4)
    echo -e "${BLUE}Launching Neural API Learning...${NC}"
    ../target/release/petaltongue ui \
      --scenario scenarios/neural-api-test.json \
      --mode neural
    ;;
  5)
    echo -e "${BLUE}Starting Full Tour...${NC}"
    echo ""
    echo -e "${YELLOW}Tour Sequence:${NC}"
    echo "  → Live Ecosystem (30s)"
    echo "  → Graph Builder (30s)"
    echo "  → RootPulse (30s)"
    echo "  → Neural Learning (30s)"
    echo ""
    
    # Live Ecosystem
    echo -e "${GREEN}[1/4] Live Ecosystem${NC}"
    timeout 30s ../target/release/petaltongue ui \
      --scenario scenarios/live-ecosystem.json \
      --mode live || true
    
    # Graph Builder
    echo -e "${GREEN}[2/4] Graph Builder Studio${NC}"
    timeout 30s ../target/release/petaltongue ui \
      --scenario scenarios/graph-studio.json \
      --mode graph-builder || true
    
    # RootPulse
    echo -e "${GREEN}[3/4] RootPulse Visualization${NC}"
    timeout 30s ../target/release/petaltongue ui \
      --scenario scenarios/rootpulse-demo.json \
      --mode rootpulse || true
    
    # Neural Learning
    echo -e "${GREEN}[4/4] Neural API Learning${NC}"
    timeout 30s ../target/release/petaltongue ui \
      --scenario scenarios/neural-api-test.json \
      --mode neural || true
    
    echo ""
    echo -e "${GREEN}Tour complete!${NC}"
    ;;
  6)
    echo "Goodbye!"
    exit 0
    ;;
  *)
    echo -e "${YELLOW}Invalid choice. Exiting.${NC}"
    exit 1
    ;;
esac

echo ""
echo -e "${GREEN}✨ benchTop demonstration complete!${NC}"
echo ""
echo "Keyboard shortcuts:"
echo "  P - Proprioception Panel"
echo "  M - Metrics Dashboard"
echo "  G - Graph Builder"
echo "  R - RootPulse"
echo "  H - Help"
echo ""
echo "🌸 petalTongue - The Human Interface for ecoPrimals"

