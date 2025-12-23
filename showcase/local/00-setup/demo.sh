#!/usr/bin/env bash
#
# 00-setup: Setup & Verification Demo
#
# Purpose: Verify environment is ready for fermentation testing
# Duration: ~5 minutes
# Dependencies: None (this is the foundation)

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PINK='\033[1;35m'
NC='\033[0m' # No Color

# Directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PETALTONGUE_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BIOMEOS_DIR="$(cd "$PETALTONGUE_DIR/../biomeOS" && pwd)"

echo -e "${PINK}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PINK}║                  🌸 petalTongue Setup & Verification                        ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Check Prerequisites
echo -e "${BLUE}📋 Step 1/5: Checking Prerequisites...${NC}"
"$SCRIPT_DIR/check-prerequisites.sh"
echo ""

# Step 2: Launch BiomeOS
echo -e "${BLUE}🚀 Step 2/5: Launching BiomeOS...${NC}"
"$SCRIPT_DIR/launch-biomeos.sh"
echo ""

# Step 3: Verify BiomeOS
echo -e "${BLUE}🔍 Step 3/5: Verifying BiomeOS...${NC}"
"$SCRIPT_DIR/verify-biomeos.sh"
echo ""

# Step 4: Launch petalTongue
echo -e "${BLUE}🌸 Step 4/5: Launching petalTongue...${NC}"
"$SCRIPT_DIR/launch-petaltongue.sh"
echo ""

# Step 5: Validate Connection
echo -e "${BLUE}✅ Step 5/5: Validating Connection...${NC}"
"$SCRIPT_DIR/validate-connection.sh"
echo ""

# Success!
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                     ✅ Setup Complete!                                        ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ BiomeOS is running on http://localhost:3000${NC}"
echo -e "${GREEN}✅ petalTongue UI is open and connected${NC}"
echo -e "${GREEN}✅ Environment is ready for fermentation testing${NC}"
echo ""
echo -e "${YELLOW}📖 Next Steps:${NC}"
echo -e "  1. Check the petalTongue window"
echo -e "  2. You should see an empty graph (no primals yet) or any running primals"
echo -e "  3. Try launching a primal: ${BLUE}$BIOMEOS_DIR/bin/primals/beardog &${NC}"
echo -e "  4. Watch it appear in petalTongue (auto-refresh every 5s)"
echo -e ""
echo -e "${YELLOW}📚 Continue Fermentation:${NC}"
echo -e "  cd ../01-single-primal/"
echo -e "  cat README.md"
echo -e ""
echo -e "${PINK}🌱 Fermentation has begun! Let petalTongue grow! 🌸${NC}"

