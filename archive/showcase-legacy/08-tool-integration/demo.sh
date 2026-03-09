#!/usr/bin/env bash
# Demo: Tool Integration - Dynamic Extensions
# Description: Integrate external tools at runtime via capability discovery
# Duration: 10 minutes
# Prerequisites: petalTongue built

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

DEMO_NAME="Tool Integration - Dynamic Extensions"
DEMO_DURATION="10 minutes"
REQUIRE_PETALTONGUE="true"
REQUIRE_BIOMEOS="false"

clear
print_header "$DEMO_NAME"
print_info "Duration: $DEMO_DURATION"
print_info "What you'll learn: Runtime tool discovery and capability-based integration"
echo

print_version_info
check_prerequisites

step 1 "Understanding Dynamic Integration"
cat << EOF
petalTongue integrates tools DYNAMICALLY:

THE PRINCIPLE:
  "Discover capabilities at runtime, never hardcode"

HOW IT WORKS:
  1. Tool announces capability ("I can visualize")
  2. petalTongue discovers ("Found viz capability")
  3. User activates ("Use this tool")
  4. Integration happens (runtime, dynamic)

CONTRAST WITH TRADITIONAL:
  ✗ Hardcoded: List of known tools compiled in
  ✓ Dynamic: Discover what's actually available

BENEFITS:
  • No hardcoded tool list
  • No compiled-in ports/addresses
  • No assumed locations
  • Graceful when tools missing
  • Easy to extend

This is EXTENSIBLE ARCHITECTURE!
EOF
pause
wait_for_user

step 2 "Built-In Tool Panels"
cat << EOF
petalTongue includes several tool integrations:

SYSTEM MONITOR:
  • CPU, memory, disk usage
  • Capability: "system.monitor"
  • Uses: sysinfo (cross-platform)

PROCESS VIEWER:
  • Running processes
  • Capability: "process.view"
  • Uses: sysinfo (cross-platform)

GRAPH METRICS:
  • Node/edge counts, density
  • Capability: "graph.metrics"
  • Uses: petgraph analysis

BINGOCUBE (if available):
  • 3D visualization
  • Capability: "bingocube.viz"
  • Uses: BingoCube adapters

TOADSTOOL (if available):
  • Python tool execution
  • Capability: "toadstool.exec"
  • Uses: ToadStool integration

Each discovered at RUNTIME!
EOF
pause
wait_for_user

step 3 "Launching petalTongue - Explore tools"
print_info "The window will open with discovered tools"
echo

print_info "DO THIS:"
print_info "  1. Look for 'Tools' panel or sidebar"
print_info "  2. See which tools were discovered"
print_info "  3. Open 'System Monitor' panel"
print_info "  4. Watch real-time CPU/memory"
print_info "  5. Try other available tools"
echo

print_info "OBSERVE:"
print_info "  • Only available tools shown"
print_info "  • Each declares its capability"
print_info "  • Missing tools = Hidden (not errors)"
print_info "  • Clean, graceful UX"
echo

print_info "KEY CONCEPTS:"
print_info "  • Runtime discovery (not compile-time)"
print_info "  • Capability-based (not name-based)"
print_info "  • Extensible (easy to add new tools)"
print_info "  • Robust (graceful degradation)"
echo

print_warning "Press Ctrl+C when done exploring"
echo

cd "${SCRIPT_DIR}/../../../"
MOCK_MODE=true cargo run --release

echo
print_success "🎉 Phase 1 Complete! You've mastered local petalTongue capabilities!"
echo

demo_complete "cd ../../02-biomeos-integration/ && cat README.md"

print_info "Phase 1 Summary - What you mastered:"
print_info "  ✓ 00: Basic visualization (hello-petaltongue)"
print_info "  ✓ 01: Graph engine (layout algorithms)"
print_info "  ✓ 02: Visual 2D (interactive controls)"
print_info "  ✓ 03: Audio sonification (multi-modal)"
print_info "  ✓ 04: Animation flow (informational motion)"
print_info "  ✓ 05: Dual modality (universal design)"
print_info "  ✓ 06: Capability detection (self-awareness)"
print_info "  ✓ 07: Audio export (pure Rust WAV)"
print_info "  ✓ 08: Tool integration (extensible)"
echo

print_success "You understand petalTongue's local capabilities deeply!"
print_info "Next: Learn how petalTongue integrates with BiomeOS and other primals"

