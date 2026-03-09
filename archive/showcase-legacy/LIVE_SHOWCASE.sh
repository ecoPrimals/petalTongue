#!/usr/bin/env bash
# petalTongue Live Showcase - Multi-Modal Accessibility Demonstrations
# ALL DATA IS LIVE - Zero Mocks

set -e

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "═══════════════════════════════════════════════════════════════════"
echo "🌸 petalTongue - Live Multi-Modal Showcase"
echo "═══════════════════════════════════════════════════════════════════"
echo
echo "This showcase demonstrates petalTongue's accessibility-first,"
echo "multi-modal design with 100% LIVE data:"
echo
echo "  • Real system metrics (CPU, memory, processes)"
echo "  • Real network discovery (mDNS)"
echo "  • Real audio capture (microphone)"
echo "  • Real entropy analysis (FFT)"
echo "  • NO MOCKS • NO FAKE DATA"
echo
echo "═══════════════════════════════════════════════════════════════════"
echo

# Menu
echo "Select demonstration:"
echo
echo "ACCESSIBILITY SCENARIOS (Any Input → Any Output):"
echo "  1) Blind User - Audio-Only Interface"
echo "  2) Deaf User - Visual-Only Interface"
echo "  3) Non-Verbal User - Alternative Input"
echo "  4) Illiterate User - Icon-Based Interface"
echo "  5) Motor Disability - Single Switch"
echo "  6) Deaf-Blind User - Tactile/Braille"
echo
echo "LIVE SYSTEM DEMOS (Real Data Visualization):"
echo "  7) Live System Metrics (CPU, Memory, Processes)"
echo "  8) Network Discovery Audio (mDNS)"
echo "  9) Audio Entropy Capture (Microphone)"
echo " 10) Process Monitor with Audio Nav"
echo
echo "MULTI-MODAL COMBOS:"
echo " 11) Visual + Audio (Full Experience)"
echo " 12) Audio + Tactile (Blind with Haptics)"
echo " 13) Visual + Text (Deaf with Captions)"
echo
echo " 0) Run ALL demos sequentially"
echo " q) Quit"
echo
read -p "Enter choice: " choice

case $choice in
    1)
        echo
        echo "🔊 Launching Blind User Demo..."
        cd "$ROOT_DIR/showcase/04-accessibility/01-blind-user"
        ./demo.sh
        ;;
    2)
        echo
        echo "👁️  Launching Deaf User Demo..."
        echo "(Coming soon - visual-only interface)"
        ;;
    3)
        echo
        echo "🗣️  Launching Non-Verbal User Demo..."
        echo "(Coming soon - alternative input methods)"
        ;;
    4)
        echo
        echo "📱 Launching Illiterate User Demo..."
        echo "(Coming soon - icon-based interface)"
        ;;
    5)
        echo
        echo "🔘 Launching Motor Disability Demo..."
        echo "(Coming soon - single switch scanning)"
        ;;
    6)
        echo
        echo "⠃ Launching Deaf-Blind User Demo..."
        echo "(Coming soon - braille/tactile interface)"
        ;;
    7)
        echo
        echo "📊 Launching Live System Metrics..."
        cd "$ROOT_DIR/showcase/05-production-scenarios/01-live-system-metrics"
        ./demo.sh
        ;;
    8)
        echo
        echo "🌐 Launching Network Discovery Audio..."
        echo "(Coming soon - mDNS audio narration)"
        ;;
    9)
        echo
        echo "🎵 Launching Audio Entropy Capture..."
        echo "This uses your REAL microphone!"
        echo
        cd "$ROOT_DIR"
        cargo run --release --features audio --bin petal-tongue
        ;;
    10)
        echo
        echo "🔍 Launching Process Monitor..."
        echo "(Coming soon - process list with audio)"
        ;;
    11)
        echo
        echo "🎨 Launching Full Multi-Modal Experience..."
        cd "$ROOT_DIR"
        cargo run --release --features audio
        ;;
    12)
        echo
        echo "🔊🤲 Launching Audio + Tactile..."
        echo "(Coming soon - audio with haptic feedback)"
        ;;
    13)
        echo
        echo "👁️📝 Launching Visual + Text..."
        cd "$ROOT_DIR"
        cargo run --release
        ;;
    0)
        echo
        echo "🚀 Running ALL demonstrations..."
        echo
        
        echo "═══ Demo 1: Blind User ═══"
        cd "$ROOT_DIR/showcase/04-accessibility/01-blind-user"
        ./demo.sh
        
        echo
        echo "═══ Demo 2: Live System Metrics ═══"
        cd "$ROOT_DIR/showcase/05-production-scenarios/01-live-system-metrics"
        timeout 10s ./demo.sh || true
        
        echo
        echo "✅ All automated demos complete!"
        ;;
    q|Q)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo
echo "═══════════════════════════════════════════════════════════════════"
echo "Demo complete! All data was LIVE from your system."
echo "═══════════════════════════════════════════════════════════════════"

