#!/usr/bin/env bash
# Demo: Graph Engine Fundamentals
# Description: Master the core graph engine and 4 layout algorithms
# Duration: 10 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

# Configuration
DEMO_NAME="Graph Engine Fundamentals - Layout Algorithms"
DEMO_DURATION="10 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

# Header
clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: The modality-agnostic graph engine + 4 layouts"
echo

print_version_info
check_prerequisites

# Demo introduction
step 1 "Understanding the Graph Engine Architecture"
cat << EOF
The graph engine is the CORE of petalTongue:

  GraphEngine (Core - Modality Agnostic)
  ├── Manages nodes and edges
  ├── Computes layout positions
  └── NO knowledge of rendering
           ↓
      ┌────┴────┐
      ↓         ↓
  Visual 2D   Audio
  Renderer   Renderer

This separation enables multi-modal rendering!
EOF
pause
wait_for_user

step 2 "The Four Layout Algorithms"
cat << EOF
petalTongue provides 4 layout algorithms:

1. FORCE-DIRECTED (Physics-based)
   • Natural clustering
   • Connected nodes near each other
   • Iterative convergence

2. CIRCULAR (Ring arrangement)
   • Perfect symmetry
   • Equal spacing
   • Instant computation

3. HIERARCHICAL (Tree structure)
   • Parent-child relationships
   • Clear organization
   • Top-down flow

4. RANDOM (Scattered)
   • Random positions
   • Useful for debugging
   • Instant computation

You'll see ALL of these in action!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Try each layout"
print_info "The window will open with a 5-node graph"
echo

print_info "TRY THIS in the UI:"
print_info "  1. Use Layout dropdown → Select 'Force-Directed'"
print_info "  2. Watch nodes arrange naturally (100 iterations)"
print_info "  3. Switch to 'Circular' → See perfect circle"
print_info "  4. Try 'Hierarchical' → See tree structure"
print_info "  5. Try 'Random' → See chaos"
echo

print_info "OBSERVE:"
print_info "  • Same graph, different positions"
print_info "  • Engine computes positions only"
print_info "  • Renderers use those positions"
print_info "  • Modality-agnostic design proven!"
echo

print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

# Demo complete
echo
demo_complete "cd ../02-visual-2d/ && cat README.md"

print_info "What you learned:"
print_info "  ✓ Graph engine is modality-agnostic"
print_info "  ✓ 4 layout algorithms available"
print_info "  ✓ Positions computed separately from rendering"
print_info "  ✓ Same data, different views possible"
echo

print_success "You understand the core architecture!"

