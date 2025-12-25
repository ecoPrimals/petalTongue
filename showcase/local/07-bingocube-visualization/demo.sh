#!/usr/bin/env bash
# BingoCube Visualization Demo - Primal Tool Use
# Demonstrates how petalTongue uses BingoCube as an external tool

set -e

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║                                                                  ║"
echo "║        🎲 BingoCube Tool Use Demonstration 🎲                   ║"
echo "║                                                                  ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "This demo shows petalTongue using BingoCube as an external tool."
echo ""
echo "Key Concepts:"
echo "  • BingoCube is a standalone cryptographic tool"
echo "  • petalTongue uses bingocube-adapters to render it"
echo "  • This is 'primal tool use' - not primal-to-primal interaction"
echo "  • Any primal can use BingoCube the same way"
echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must run from petalTongue root directory"
    echo "   cd to petalTongue root and run: showcase/local/07-bingocube-visualization/demo.sh"
    exit 1
fi

echo "📦 Building petalTongue with BingoCube integration..."
cargo build --release -p petal-tongue-ui

echo ""
echo "✅ Build complete!"
echo ""
echo "🚀 Starting petalTongue..."
echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "INSTRUCTIONS:"
echo "═══════════════════════════════════════════════════════════════════"
echo ""
echo "1. Click '🎲 BingoCube Tool' in the top menu bar"
echo "2. Enter a seed (try: 'alice', 'bob', 'test-identity')"
echo "3. Adjust the reveal slider (0-100%)"
echo "4. Click 'Generate New' to create a new pattern"
echo "5. Toggle back to graph view anytime"
echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# Run the application
cargo run --release -p petal-tongue-ui
