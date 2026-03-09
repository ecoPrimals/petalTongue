#!/usr/bin/env bash
# Demo: Audio Export - Sonification to File
# Description: Export graph sonifications as audio files
# Duration: 5 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Audio Export - Sonification to File"
DEMO_DURATION="5 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Pure Rust audio file generation"
echo

print_version_info
check_prerequisites

step 1 "Understanding Audio Export"
cat << EOF
petalTongue can export graph sonifications as audio files:

WORKFLOW:
  Graph State → Sonification → WAV File

FILE FORMAT:
  • Format: WAV (uncompressed)
  • Sample Rate: 44,100 Hz (CD quality)
  • Bit Depth: 16-bit
  • Channels: 2 (stereo)
  • Generator: Pure Rust (no system libs!)

WHY EXPORT?
  1. Offline playback (media players)
  2. Analysis (spectrograms, ML)
  3. Accessibility (standard format)
  4. Documentation (audio logs)

Pure Rust = Cross-platform + No dependencies!
EOF
pause
wait_for_user

step 2 "Pure Rust Audio Generation"
cat << EOF
Why pure Rust matters:

NO SYSTEM DEPENDENCIES:
  ✗ No ALSA (Linux audio system)
  ✗ No CoreAudio (macOS)
  ✗ No WASAPI (Windows)
  ✓ Pure Rust WAV generation

BENEFITS:
  • Cross-platform (works everywhere)
  • No build dependencies
  • No runtime dependencies
  • Predictable behavior
  • Never breaks from system updates

TRADEOFFS:
  • Export only (no real-time playback)
  • WAV format (uncompressed, larger)
  • But: Universal compatibility!

This is SOVEREIGNTY - no system gatekeepers!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Export audio"
print_info "The window will open with audio export available"
echo

print_info "DO THIS:"
print_info "  1. Open 'Audio Export' panel (or menu)"
print_info "  2. Set duration (default: 5 seconds)"
print_info "  3. Click 'Export WAV'"
print_info "  4. Choose save location"
echo

print_info "THEN:"
print_info "  5. Open exported file in media player"
print_info "  6. Listen to the sonification"
print_info "  7. Observe: Same as real-time audio!"
echo

print_info "FILE CONTAINS:"
print_info "  • All instruments (primal types)"
print_info "  • Stereo positioning (spatial)"
print_info "  • Pitch quality (health states)"
print_info "  • Snapshot of system state in audio!"
echo

print_warning "Note: Export feature may be in development"
print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
demo_complete "cd ../08-tool-integration/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ Pure Rust WAV generation (no dependencies)"
print_info "  ✓ Cross-platform audio export"
print_info "  ✓ Offline sonification playback"
print_info "  ✓ Standard format (universal access)"
print_info "  ✓ Audio documentation capability"
echo

print_success "You understand sovereign audio generation!"

