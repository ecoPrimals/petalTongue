#!/usr/bin/env bash
# Demo: Hello petalTongue
# Description: Your first visualization - multi-modal graph rendering
# Duration: 5 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# Configuration
DEMO_NAME="Hello petalTongue - Your First Visualization"
DEMO_DURATION="5 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

# Header
clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll see: A simple 3-node graph with visual + audio"
echo

# Version info
print_version_info

# Prerequisites
check_prerequisites

# Demo introduction
step 1 "Understanding what you're about to see"
cat << EOF
petalTongue will open showing:

VISUAL:
  - 3 colored nodes (Discovery, Security, Storage)
  - 2 edges connecting them
  - Interactive controls (zoom, pan, layouts)

AUDIO (if enabled):
  - 3 distinct instrument tones
  - Spatial positioning (left/center/right)
  - Harmonic sounds (healthy state)

This proves: Same information, different sensory channels!
EOF
pause
wait_for_user

# Launch petalTongue
step 2 "Launching petalTongue with mock data"
print_info "Starting petalTongue..."
print_info "A window will open - explore the visualization"
echo

print_info "Try these interactions:"
print_info "  • Mouse wheel → Zoom in/out"
print_info "  • Click + drag → Pan around"
print_info "  • Layout dropdown → Try different algorithms"
print_info "  • Click nodes → Select them"
print_info "  • Enable Audio → Hear the soundscape"
echo

print_warning "Press Ctrl+C in the window to close when done exploring"
echo

# Run petalTongue with mock mode
cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

# Demo complete
echo
demo_complete "cd ../01-graph-engine/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ petalTongue launches and works"
print_info "  ✓ Visual + Audio modalities proven"
print_info "  ✓ Interactive exploration possible"
print_info "  ✓ Multi-modal design validated"
echo

print_success "You're ready for more advanced demos!"

