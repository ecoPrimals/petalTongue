#!/usr/bin/env bash
# Demo: Dual Modality - Visual AND Audio Together
# Description: Experience the power of multi-modal design
# Duration: 10 minutes
# Prerequisites: petalTongue built, audio recommended

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Dual Modality - Visual AND Audio Together"
DEMO_DURATION="10 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Multi-modal design as universal design"
echo

print_version_info
check_prerequisites

step 1 "Understanding Multi-Modal Design"
cat << EOF
petalTongue provides the SAME information via BOTH modalities:

INFORMATION → VISUAL → AUDIO
─────────────────────────────
Primal Type → Color → Instrument
Health → Shape → Pitch quality
Position → X/Y → Stereo pan
Activity → Pulse → Rhythm/tempo
Bandwidth → Thickness → Volume

NOT "visual with audio support"
→ Equal, first-class modalities!

This is UNIVERSAL DESIGN:
• Blind users → Complete via audio
• Deaf users → Complete via visual
• All users → Enhanced with both

No user left behind!
EOF
pause
wait_for_user

step 2 "Why Both Modalities?"
cat << EOF
Four powerful reasons:

1. REDUNDANCY = ROBUSTNESS
   → Multiple channels = No single failure point
   → Vision impaired? Audio complete!
   → Hearing impaired? Visual complete!

2. CROSS-SENSORY VALIDATION
   → "I see 5 nodes" + "I hear 5 instruments" = Confirmed
   → Visual warning + Audio dissonance = Validated
   → Multiple senses increase confidence

3. COGNITIVE LOAD DISTRIBUTION
   → Eyes: Focus on spatial layout
   → Ears: Monitor background health
   → Brain: Integrates effortlessly
   → Each sense does what it does best

4. UNIVERSAL DESIGN = HUMAN DIGNITY
   → All users can operate independently
   → No second-class experience
   → Accessibility is capability, not limitation

This is the future!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Experience both modalities"
print_info "The window will open with both modalities available"
echo

print_info "EXPERIMENT 1: Close your eyes"
print_info "  • Can you count nodes? (instruments)"
print_info "  • Can you identify types? (timbres)"
print_info "  • Can you locate positions? (stereo)"
print_info "  • Can you detect health? (pitch)"
print_info "  → Full navigation without vision!"
echo

print_info "EXPERIMENT 2: Mute audio"
print_info "  • Can you count nodes? (visual)"
print_info "  • Can you identify types? (colors)"
print_info "  • Can you see layout? (positions)"
print_info "  • Can you spot issues? (indicators)"
print_info "  → Full navigation without audio!"
echo

print_info "EXPERIMENT 3: Use both together"
print_info "  • Eyes: Focus on topology"
print_info "  • Ears: Monitor background health"
print_info "  • Brain: Integrates seamlessly"
print_info "  → Enhanced awareness!"
echo

print_warning "This demo works best with audio enabled"
print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
demo_complete "cd ../06-capability-detection/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ Multi-modal = Equal modalities (not primary+support)"
print_info "  ✓ Redundant encoding = Robustness"
print_info "  ✓ Either sense provides complete information"
print_info "  ✓ Both together = Enhanced experience"
print_info "  ✓ Universal design = Human dignity"
echo

print_success "You understand the power of multi-modal design!"

