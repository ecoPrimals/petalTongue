#!/usr/bin/env bash
# Demo: Animation Flow - Data in Motion
# Description: Visualize data flow and activity with animations
# Duration: 10 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Animation Flow - Data in Motion"
DEMO_DURATION="10 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Animation as information, not decoration"
echo

print_version_info
check_prerequisites

step 1 "Understanding Animation Purpose"
cat << EOF
petalTongue animations convey INFORMATION:

FLOW PARTICLES:
  ●──●──●─→ Particles = Data flow
  Direction = Source → Destination
  Count = Bandwidth volume
  Speed = Transfer rate

NODE PULSES:
  ● Idle = No pulse
  ⦿ Active = Slow pulse
  ◉ Busy = Fast pulse

EDGE ANIMATIONS:
  ──── Static = No activity
  ●──● Dashed = Flowing data
  ==== Thick = High bandwidth

Every animation has MEANING!
EOF
pause
wait_for_user

step 2 "Animation Design Principles"
cat << EOF
Our animation philosophy:

1. PURPOSEFUL, NOT DECORATIVE
   → Every animation conveys information
   → No "eye candy" without meaning
   
2. PERFORMANCE AWARE
   → Scales with system size
   → Always 30+ FPS responsive

3. ACCESSIBILITY CONSCIOUS
   → Can be disabled (motion sensitivity)
   → Has audio equivalent
   → Simplified modes available

4. MULTI-MODAL CONSISTENT
   → Visual: Particles + pulses
   → Audio: Rhythm + tempo changes
   → Same information, different channels

Animation serves the user, not ego!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Observe activity"
print_info "The window will open with animations available"
echo

print_info "DO THIS:"
print_info "  1. Check 'Enable Animation' checkbox"
print_info "  2. Watch for flow particles (dots on edges)"
print_info "  3. Observe node pulses (breathing effect)"
print_info "  4. Toggle animation on/off to compare"
echo

print_info "LOOK FOR PATTERNS:"
print_info "  • Heavy traffic → Many particles"
print_info "  • Busy nodes → Fast pulses"
print_info "  • Idle nodes → No pulses"
print_info "  • Bottlenecks → Particles accumulating"
echo

print_info "COMPARE MODES:"
print_info "  • Animation OFF → Static graph"
print_info "  • Animation ON → Activity visible"
print_info "  Which reveals more about system state?"
echo

print_warning "Note: Animations auto-scale for performance"
print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
demo_complete "cd ../05-dual-modality/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ Animations convey information (not decoration)"
print_info "  ✓ Flow particles show data movement"
print_info "  ✓ Node pulses indicate activity"
print_info "  ✓ Patterns reveal system health"
print_info "  ✓ Performance scales gracefully"
echo

print_success "You understand informational animation!"

