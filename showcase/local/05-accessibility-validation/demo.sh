#!/usr/bin/env bash
#
# 05-accessibility-validation: Universal Accessibility Testing
#
# Purpose: Validate audio-only, screen reader, keyboard, colorblind accessibility
# Duration: ~20-30 minutes

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PINK='\033[1;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${PINK}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PINK}║                🌸 Universal Accessibility Validation                        ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo validates that petalTongue is accessible to ALL users.${NC}"
echo -e "${BLUE}We'll test audio-only mode, screen readers, keyboard navigation, and more!${NC}"
echo ""

echo -e "${YELLOW}📋 Prerequisites:${NC}"
echo -e "   • Headphones (recommended for audio testing)"
echo -e "   • Screen reader installed (orca/VoiceOver/NVDA)"
echo -e "   • Willingness to close your eyes and trust audio"
echo ""
echo -e "${YELLOW}Press Enter to begin accessibility testing...${NC}"
read -r

# Phase 1: Audio-Only Mode
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 1: Audio-Only Mode (Blind User Simulation)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🎵 Objective: Can you understand the ecosystem using ONLY audio?${NC}"
echo ""
echo -e "${YELLOW}Instructions:${NC}"
echo -e "  1. Put on headphones"
echo -e "  2. Look at petalTongue UI (for now)"
echo -e "  3. Enable Audio mode in UI"
echo -e "  4. Listen to the soundscape"
echo -e "  5. Click 'AI Narration' button to hear description"
echo ""
echo -e "${YELLOW}Try to answer (from audio alone):${NC}"
echo -e "  • How many primals are running?"
echo -e "  • Which types are present?"
echo -e "  • What are their health states?"
echo -e "  • Are there connections?"
echo ""
echo -e "${YELLOW}Press Enter when ready for eyes-closed test...${NC}"
read -r

echo ""
echo -e "${CYAN}👁️ Now CLOSE YOUR EYES (or turn away from screen)${NC}"
echo -e "${YELLOW}Rely ONLY on audio. Can you still understand the ecosystem?${NC}"
echo ""
echo -e "${YELLOW}Press Enter after 30 seconds of audio-only observation...${NC}"
read -r

echo ""
echo -e "${YELLOW}❓ Reflection:${NC}"
echo -e "  • Could you identify primals by sound?"
echo -e "  • Was AI narration accurate and helpful?"
echo -e "  • Did sonification make sense (pitch = health, position = stereo)?"
echo -e "  • Would you trust this in production?"
echo ""
echo -e "${YELLOW}Document any gaps in: ../GAPS.md${NC}"
echo ""
echo -e "${YELLOW}Press Enter to proceed to screen reader testing...${NC}"
read -r

# Phase 2: Screen Reader
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 2: Screen Reader Compatibility${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🗣️ Objective: Can a screen reader user navigate petalTongue?${NC}"
echo ""
echo -e "${YELLOW}Instructions:${NC}"
echo -e "  1. Launch screen reader:"
echo -e "     • Linux: orca"
echo -e "     • Mac: VoiceOver (Cmd+F5)"
echo -e "     • Windows: NVDA or JAWS"
echo -e ""
echo -e "  2. Navigate petalTongue using Tab/Shift+Tab"
echo -e "  3. Try to:"
echo -e "     • Click Refresh button"
echo -e "     • Change layout dropdown"
echo -e "     • Select a node"
echo -e "     • Read stats panel"
echo ""
echo -e "${YELLOW}Validation:${NC}"
echo -e "  • Are all elements announced?"
echo -e "  • Is tab order logical?"
echo -e "  • Can you operate without mouse?"
echo ""
echo -e "${YELLOW}Press Enter when done with screen reader testing...${NC}"
read -r

# Phase 3: Keyboard Navigation
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 3: Keyboard-Only Navigation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}⌨️  Objective: Can you use petalTongue without a mouse?${NC}"
echo ""
echo -e "${YELLOW}Keyboard Map (test each):${NC}"
echo -e "  Tab           - Next element"
echo -e "  Shift+Tab     - Previous element"
echo -e "  Enter/Space   - Activate button"
echo -e "  Arrow Keys    - Pan graph (if implemented)"
echo -e "  +/-           - Zoom (if implemented)"
echo -e "  R             - Reset camera (if implemented)"
echo -e "  L             - Cycle layouts (if implemented)"
echo ""
echo -e "${YELLOW}Test:${NC}"
echo -e "  • Navigate entire UI with keyboard"
echo -e "  • Change layout"
echo -e "  • Refresh graph"
echo -e "  • Select node (if possible)"
echo ""
echo -e "${YELLOW}Questions:${NC}"
echo -e "  • Are focus indicators visible?"
echo -e "  • Is tab order logical?"
echo -e "  • Are shortcuts discoverable?"
echo ""
echo -e "${YELLOW}Press Enter when done with keyboard testing...${NC}"
read -r

# Phase 4: Colorblind Mode
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 4: Color Vision Deficiency${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🎨 Objective: Can colorblind users distinguish health states?${NC}"
echo ""
echo -e "${YELLOW}Context:${NC}"
echo -e "  • 8% of males have color vision deficiency"
echo -e "  • Red-green colorblindness most common"
echo -e "  • Current: Green=Healthy, Yellow=Warning, Red=Critical"
echo ""
echo -e "${YELLOW}Test:${NC}"
echo -e "  • Squint or use colorblind simulator"
echo -e "  • Can you still distinguish health states?"
echo ""
echo -e "${YELLOW}Alternatives (not yet implemented):${NC}"
echo -e "  • Shapes: Circle=Healthy, Triangle=Warning, Square=Critical"
echo -e "  • Patterns: Solid, Striped, Dotted"
echo -e "  • Sizes: Small=Healthy, Large=Critical"
echo -e "  • Text labels always visible"
echo ""
echo -e "${YELLOW}Document if color-only indicators are insufficient!${NC}"
echo ""
echo -e "${YELLOW}Press Enter to proceed to cognitive accessibility...${NC}"
read -r

# Phase 5: Cognitive Accessibility
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 5: Cognitive Accessibility${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🧠 Objective: Is petalTongue usable for neurodiverse users?${NC}"
echo ""
echo -e "${YELLOW}Considerations:${NC}"
echo -e "  • ADHD: Is visual complexity overwhelming?"
echo -e "  • Dyslexia: Is text readable?"
echo -e "  • Autism: Does audio cause sensory overload?"
echo -e "  • Anxiety: Are red colors stressful?"
echo ""
echo -e "${YELLOW}Questions:${NC}"
echo -e "  • Can you simplify the UI?"
echo -e "  • Can you disable audio?"
echo -e "  • Can you slow down animations?"
echo -e "  • Are there too many simultaneous updates?"
echo -e "  • Is information prioritized (most important first)?"
echo ""
echo -e "${YELLOW}Reflection:${NC}"
echo -e "  • Would you be comfortable using this for 8 hours?"
echo -e "  • Does it cause fatigue?"
echo -e "  • Is cognitive load manageable?"
echo ""
echo -e "${YELLOW}Press Enter to finish accessibility validation...${NC}"
read -r

# Summary
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                ✅ Accessibility Validation Complete!                         ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've tested audio-only mode${NC}"
echo -e "${GREEN}✅ You've validated screen reader compatibility${NC}"
echo -e "${GREEN}✅ You've tried keyboard-only navigation${NC}"
echo -e "${GREEN}✅ You've considered colorblind users${NC}"
echo -e "${GREEN}✅ You've evaluated cognitive accessibility${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Accessibility = Better UX for everyone"
echo -e "  • Audio-only must be complete (not a gimmick)"
echo -e "  • Screen readers require semantic HTML + ARIA"
echo -e "  • Keyboard navigation must be complete"
echo -e "  • Color alone is insufficient"
echo -e "  • Cognitive load matters"
echo ""
echo -e "${YELLOW}🔍 Critical Gaps to Document:${NC}"
echo -e "  • Audio-only completeness"
echo -e "  • Screen reader coverage"
echo -e "  • Keyboard shortcuts missing"
echo -e "  • Color dependency"
echo -e "  • Cognitive overload"
echo -e "  • Document ALL in: ../GAPS.md"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../06-performance-benchmarking/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Accessibility matters! Moving to performance testing! 🌸${NC}"

