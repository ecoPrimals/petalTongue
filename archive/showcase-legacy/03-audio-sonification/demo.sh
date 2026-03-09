#!/usr/bin/env bash
# Demo: Audio Sonification - Sound as Information
# Description: Experience graph data through spatial audio
# Duration: 10 minutes
# Prerequisites: petalTongue built, audio output recommended

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Audio Sonification - Sound as Information"
DEMO_DURATION="10 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Audio as a first-class information modality"
echo

print_version_info
check_prerequisites

step 1 "Understanding Audio Encoding"
cat << EOF
petalTongue uses audio encoding to convey information:

INSTRUMENTS = Primal Type:
  🎐 Chimes → Discovery (Songbird) - Bright, exploratory
  🎸 Bass → Security (BearDog) - Deep, foundational
  🎻 Strings → Storage (NestGate) - Sustained, persistent
  🥁 Drums → Compute (ToadStool) - Rhythmic, active
  🎹 Synth → AI (Squirrel) - Electronic, intelligent

PITCH = Health State:
  Harmonic → Healthy (pleasant)
  Off-key → Warning (noticeable)
  Dissonant → Critical (urgent)

STEREO = Position:
  Left speaker → Left nodes
  Right speaker → Right nodes
  Both speakers → Center nodes

Same information as visual, different channel!
EOF
pause
wait_for_user

step 2 "Why These Mappings?"
cat << EOF
The instrument choices are INTUITIVE:

🎸 Security = Bass
  → Foundation of the system
  → Grounding, stable, deep
  
🎐 Discovery = Chimes  
  → Exploring, finding, light
  → High-pitched, bright, airy

🎻 Storage = Strings
  → Persistent, flowing, sustained
  → Continuous presence

🥁 Compute = Drums
  → Activity, processing, rhythm
  → Percussive, dynamic

🎹 AI = Synth
  → Intelligence, adaptation
  → Electronic, futuristic

These aren't random - they make CONCEPTUAL SENSE!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Listen to the soundscape"
print_info "The window will open with audio available"
echo

print_info "DO THIS:"
print_info "  1. Check 'Enable Audio' checkbox"
print_info "  2. Adjust volume if needed"
print_info "  3. CLOSE YOUR EYES"
print_info "  4. Listen to the soundscape"
echo

print_info "CAN YOU HEAR:"
print_info "  • Different instruments? (5 types)"
print_info "  • Spatial positioning? (left/center/right)"
print_info "  • Harmonic tones? (all healthy)"
print_info "  • Blending? (multiple nodes = harmony)"
echo

print_info "TRY THIS:"
print_info "  • Identify each instrument type"
print_info "  • Map node positions by sound alone"
print_info "  • Detect which nodes are left/right"
print_info "  • Listen for health state (pitch quality)"
echo

print_warning "Note: Audio is optional but recommended for this demo"
print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
demo_complete "cd ../04-animation-flow/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ Audio as first-class modality (not afterthought)"
print_info "  ✓ 5 intuitive instrument mappings"
print_info "  ✓ Spatial positioning through stereo"
print_info "  ✓ Health states through pitch"
print_info "  ✓ Blind users can navigate graphs!"
echo

print_success "You understand universal design!"

