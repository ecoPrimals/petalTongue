#!/usr/bin/env bash
set -euo pipefail

# Showcase 08: Pure Rust Audio Export Demo
# Demonstrates self-aware audio capability detection and WAV export

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

cd "$PROJECT_ROOT" || exit 1

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                              ║${NC}"
echo -e "${BLUE}║          Showcase 08: Pure Rust Audio Export                                 ║${NC}"
echo -e "${BLUE}║                                                                              ║${NC}"
echo -e "${BLUE}║          Self-Aware System + Multi-Modal Representation                      ║${NC}"
echo -e "${BLUE}║                                                                              ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if petalTongue builds
echo -e "${YELLOW}[1/5]${NC} Verifying build..."
if cargo build --release -p petal-tongue-ui --quiet 2>&1 | grep -q "error"; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Run tests
echo -e "${YELLOW}[2/5]${NC} Running audio export tests..."
if cargo test -p petal-tongue-graph audio_export --quiet 2>&1 | grep -q "FAILED"; then
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ All audio export tests passing${NC}"
echo ""

# Run capability tests
echo -e "${YELLOW}[3/5]${NC} Running capability detection tests..."
if cargo test -p petal-tongue-core capability --quiet 2>&1 | grep -q "FAILED"; then
    echo -e "${RED}✗ Tests failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ All capability tests passing${NC}"
echo ""

# Create audio_export directory if needed
mkdir -p audio_export

echo -e "${YELLOW}[4/5]${NC} Launching petalTongue..."
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}                    petalTongue is now running!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}🎯 INTERACTIVE FEATURES TO TEST:${NC}"
echo ""
echo -e "  ${YELLOW}1. Self-Aware Capabilities${NC}"
echo "     • Check '🔍 Capabilities' checkbox in top menu"
echo "     • See honest status for all 6 modalities"
echo "     • Audio shows 'Unavailable (tested)' - the system KNOWS"
echo ""
echo -e "  ${YELLOW}2. Graph Audio Export${NC}"
echo "     • Check 'Audio Info' panel on the right"
echo "     • Scroll to '💾 Export Audio' section"
echo "     • Click '💾 Export Soundscape to WAV'"
echo "     • Console shows: ✅ Soundscape exported to: ..."
echo ""
echo -e "  ${YELLOW}3. BingoCube Audio Export${NC}"
echo "     • Click '🎲 BingoCube Tool' in top menu"
echo "     • Click '🎵 Audio' button"
echo "     • Scroll to '💾 Export Audio' section"
echo "     • Click '💾 Export BingoCube Soundscape'"
echo "     • Console shows export confirmation"
echo ""
echo -e "  ${YELLOW}4. Progressive Reveal Test${NC}"
echo "     • In BingoCube, adjust reveal slider (0% → 100%)"
echo "     • Export at different reveal levels"
echo "     • Compare how soundscape changes"
echo ""
echo -e "  ${YELLOW}5. Configuration Test${NC}"
echo "     • Click '⚙ Config' in BingoCube"
echo "     • Try presets: Small (5×5), Medium (8×8), Large (12×12)"
echo "     • Export each and compare complexity"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}[5/5]${NC} Starting application..."
echo ""

# Launch petalTongue
cargo run --release -p petal-tongue-ui &
PETALTONGUE_PID=$!

# Wait for user to finish testing
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Press Enter when done testing to see exported files...${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
read -r

# Kill petalTongue
kill $PETALTONGUE_PID 2>/dev/null || true

# Show exported files
echo ""
echo -e "${YELLOW}Exported Audio Files:${NC}"
echo ""
if [ -d "audio_export" ] && [ "$(ls -A audio_export)" ]; then
    ls -lh audio_export/
    echo ""
    
    # Count files
    GRAPH_COUNT=$(ls audio_export/graph_soundscape_*.wav 2>/dev/null | wc -l)
    BINGO_COUNT=$(ls audio_export/bingocube_soundscape_*.wav 2>/dev/null | wc -l)
    
    echo -e "${GREEN}✓ Graph soundscapes exported: ${GRAPH_COUNT}${NC}"
    echo -e "${GREEN}✓ BingoCube soundscapes exported: ${BINGO_COUNT}${NC}"
    echo ""
    
    # Offer to play
    if command -v mpv &> /dev/null; then
        echo -e "${BLUE}Play exported audio with mpv?${NC}"
        echo -e "  ${YELLOW}1)${NC} Play all"
        echo -e "  ${YELLOW}2)${NC} Play graph soundscapes only"
        echo -e "  ${YELLOW}3)${NC} Play BingoCube soundscapes only"
        echo -e "  ${YELLOW}4)${NC} Skip"
        echo ""
        read -p "Choice (1-4): " -r PLAY_CHOICE
        
        case $PLAY_CHOICE in
            1)
                echo "Playing all exported audio..."
                mpv --volume=70 audio_export/*.wav
                ;;
            2)
                echo "Playing graph soundscapes..."
                mpv --volume=70 audio_export/graph_soundscape_*.wav
                ;;
            3)
                echo "Playing BingoCube soundscapes..."
                mpv --volume=70 audio_export/bingocube_soundscape_*.wav
                ;;
            *)
                echo "Skipped playback"
                ;;
        esac
    else
        echo -e "${YELLOW}Install mpv to play audio:${NC}"
        echo "  sudo apt-get install mpv"
        echo ""
        echo "Or play with any audio player:"
        echo "  vlc audio_export/*.wav"
    fi
else
    echo -e "${YELLOW}No files exported yet.${NC}"
    echo ""
    echo "To export audio:"
    echo "  1. Run this demo again"
    echo "  2. Click the export buttons in the UI"
    echo "  3. Files will appear in ./audio_export/"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}                       Showcase Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}Key Achievements Demonstrated:${NC}"
echo "  ✓ Self-aware capability detection"
echo "  ✓ Honest status reporting (no false claims)"
echo "  ✓ Pure Rust audio export (no system dependencies)"
echo "  ✓ Multi-modal representation (visual + audio)"
echo "  ✓ Graceful degradation (works everywhere)"
echo ""
echo -e "${BLUE}Documentation:${NC}"
echo "  • Full details: showcase/local/08-audio-export/README.md"
echo "  • Architecture: TOADSTOOL_AUDIO_INTEGRATION.md"
echo "  • Philosophy: AUDIO_INTEGRITY_REPORT.md"
echo ""
echo -e "${GREEN}\"In critical moments, honesty saves lives.\" 🌟${NC}"
echo ""

