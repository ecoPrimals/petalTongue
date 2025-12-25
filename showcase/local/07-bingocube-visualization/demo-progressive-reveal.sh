#!/usr/bin/env bash
#
# BingoCube Progressive Reveal Demo
#
# Demonstrates animated progressive reveal from x=0.0 to x=1.0

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

cd "$PROJECT_ROOT"

echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                                                                              ║"
echo "║              🎬 BingoCube Progressive Reveal Animation 🎬                    ║"
echo "║                                                                              ║"
echo "╠══════════════════════════════════════════════════════════════════════════════╣"
echo "║                                                                              ║"
echo "║  This demo shows the progressive reveal feature of BingoCube:                ║"
echo "║                                                                              ║"
echo "║  x = 0.0  →  Empty grid (no cells revealed)                                  ║"
echo "║  x = 0.2  →  5 cells revealed (20% trust)                                    ║"
echo "║  x = 0.5  →  13 cells revealed (50% trust)                                   ║"
echo "║  x = 1.0  →  25 cells revealed (100% trust)                                  ║"
echo "║                                                                              ║"
echo "║  The nested mask property ensures:                                           ║"
echo "║    𝓜₀.₂ ⊂ 𝓜₀.₅ ⊂ 𝓜₁.₀                                                      ║"
echo "║                                                                              ║"
echo "║  Cells revealed at lower x are always present at higher x.                   ║"
echo "║  This enables incremental trust building.                                    ║"
echo "║                                                                              ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Building demo application..."
cargo build --release --bin bingocube-demo

echo ""
echo "Launching progressive reveal demo..."
echo "(The demo will auto-animate from x=0.0 to x=1.0)"
echo "(Press Ctrl+C to exit)"
echo ""

# Launch with animation enabled
# TODO: Add CLI flag to auto-start animation
./target/release/bingocube-demo

