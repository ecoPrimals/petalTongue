#!/usr/bin/env bash
# Demo: Visual 2D - Interactive Visualization
# Description: Master interactive graph visualization and controls
# Duration: 10 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Visual 2D - Interactive Visualization"
DEMO_DURATION="10 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Interactive visualization and visual encoding"
echo

print_version_info
check_prerequisites

step 1 "Understanding Visual Encoding"
cat << EOF
petalTongue uses visual encoding to convey information:

COLORS = Primal Type:
  🟦 Blue → Discovery (Songbird)
  🟩 Green → Security (BearDog)  
  🟧 Orange → Storage (NestGate)
  🟣 Purple → Compute (ToadStool)
  🟪 Pink → AI (Squirrel)

SHAPES = Health State:
  ● Solid → Healthy
  ◍ Pulsing → Warning
  ◉ Flashing → Critical

THICKNESS = Bandwidth:
  ─ Thin → Low traffic
  ━ Thick → High traffic

Same information, visual channel!
EOF
pause
wait_for_user

step 2 "Interactive Controls"
cat << EOF
You can interact with the visualization:

MOUSE:
  • Scroll Wheel → Zoom in/out
  • Click + Drag → Pan around
  • Click Node → Select and inspect
  
KEYBOARD:
  • Arrow Keys → Pan direction
  • +/- → Zoom in/out
  • Space → Reset view
  • Escape → Deselect

CONTROL PANEL:
  • Layout dropdown → Change algorithm
  • Checkboxes → Toggle features
  • Sliders → Adjust parameters
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Try interactions"
print_info "The window will open with interactive controls enabled"
echo

print_info "TRY THESE INTERACTIONS:"
print_info "  1. Zoom IN (scroll up) - See node details"
print_info "  2. Zoom OUT (scroll down) - See full graph"
print_info "  3. Pan AROUND (click + drag) - Explore"
print_info "  4. Select NODE (click) - See details panel"
print_info "  5. Change LAYOUT - Watch rearrangement"
echo

print_info "OBSERVE:"
print_info "  • Colors encode primal types"
print_info "  • Shapes show health states"
print_info "  • Thickness shows bandwidth"
print_info "  • Selection shows details"
echo

print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
demo_complete "cd ../03-audio-sonification/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ Visual encoding (color, shape, size)"
print_info "  ✓ Interactive controls (zoom, pan, select)"
print_info "  ✓ Real-time visualization"
print_info "  ✓ Information at a glance"
echo

print_success "You can navigate complex graphs visually!"

