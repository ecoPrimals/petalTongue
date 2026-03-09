#!/usr/bin/env bash
# Demo: Capability Detection - Know Thyself
# Description: petalTongue discovers and reports its own capabilities
# Duration: 5 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Capability Detection - Know Thyself"
DEMO_DURATION="5 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Self-awareness and honest reporting"
echo

print_version_info
check_prerequisites

step 1 "The Prime Directive"
cat << EOF
petalTongue follows one critical principle:

  "NEVER CLAIM A CAPABILITY THAT ISN'T REAL"

This means:
• Detect, don't assume
• Report honestly, not optimistically
• Degrade gracefully when needed
• Never lie to users

Examples:
  ✓ Audio device present? → "Audio available"
  ✗ No audio device? → "Audio unavailable" (graceful)
  ✓ GPU available? → "Smooth animations"
  ✗ CPU only? → "Simple animations"

Honest communication builds trust!
EOF
pause
wait_for_user

step 2 "What Gets Detected"
cat << EOF
petalTongue detects:

MODALITIES:
  • Visual (required) → egui + display
  • Audio (optional) → Audio device
  • Haptic (future) → Not yet implemented

FEATURES:
  • Animation → GPU/CPU capability
  • BingoCube → Integration available?
  • ToadStool → Python runtime present?

SYSTEM RESOURCES:
  • CPU cores → Animation complexity
  • Memory → Graph size limits
  • Display → Layout scaling

Everything is DISCOVERED, not ASSUMED!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - View capability report"
print_info "The window will open with capability detection complete"
echo

print_info "LOOK FOR:"
print_info "  1. System Capabilities panel (or menu)"
print_info "  2. Visual status (should be ✓ Available)"
print_info "  3. Audio status (depends on your system)"
print_info "  4. Animation capability"
print_info "  5. Integration status"
echo

print_info "OBSERVE:"
print_info "  • Honest reporting (not optimistic)"
print_info "  • Graceful degradation (if needed)"
print_info "  • Clear reasons (when unavailable)"
print_info "  • System resources (detected)"
echo

print_info "KEY CONCEPT:"
print_info "  If audio shows 'unavailable', that's CORRECT!"
print_info "  petalTongue is being honest, not broken."
print_info "  It continues working perfectly via visual."
echo

print_warning "Press Ctrl+C when done reviewing"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
demo_complete "cd ../07-audio-export/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ 'Never claim false capabilities' principle"
print_info "  ✓ Runtime detection (not assumptions)"
print_info "  ✓ Honest reporting (builds trust)"
print_info "  ✓ Graceful degradation (adaptive)"
print_info "  ✓ Self-awareness (fundamental)"
echo

print_success "You understand responsible capability reporting!"

