#!/usr/bin/env bash
# Quick start script for petalTongue showcase
# Runs the first 3 demos for a fast introduction

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

clear
echo "════════════════════════════════════════════════════════════"
echo "  🌸 petalTongue Showcase - Quick Start"
echo "════════════════════════════════════════════════════════════"
echo
echo "This quick start will run 3 essential demos:"
echo "  1. Hello petalTongue (5 min) - First visualization"
echo "  2. Graph Engine (10 min) - Layout algorithms"
echo "  3. Visual 2D (10 min) - Interactive controls"
echo
echo "Total time: ~25 minutes"
echo
echo "These demos give you a solid foundation."
echo "After this, explore the full showcase at your own pace!"
echo
read -p "Press Enter to begin, or Ctrl+C to cancel..."
echo

demos=(
    "01-local-primal/00-hello-petaltongue"
    "01-local-primal/01-graph-engine"
    "01-local-primal/02-visual-2d"
)

total=${#demos[@]}
current=0

for demo in "${demos[@]}"; do
    ((current++))
    
    echo
    echo "════════════════════════════════════════════════════════════"
    echo "  Demo ${current}/${total}: ${demo}"
    echo "════════════════════════════════════════════════════════════"
    echo
    
    demo_path="${SCRIPT_DIR}/${demo}"
    
    if [[ ! -d "$demo_path" ]]; then
        echo "⚠️  Demo directory not found: $demo_path"
        echo "Skipping..."
        continue
    fi
    
    if [[ ! -f "$demo_path/demo.sh" ]]; then
        echo "⚠️  Demo script not found: $demo_path/demo.sh"
        echo "Skipping..."
        continue
    fi
    
    if cd "$demo_path" && bash demo.sh; then
        echo "✅ Demo completed"
    else
        exit_code=$?
        if [[ $exit_code -eq 130 ]]; then
            echo "⏭️  Demo interrupted, moving to next..."
        else
            echo "❌ Demo failed with exit code: $exit_code"
            exit 1
        fi
    fi
    
    if [[ $current -lt $total ]]; then
        echo
        read -p "Press Enter for next demo, or Ctrl+C to stop..."
        clear
    fi
done

echo
echo "════════════════════════════════════════════════════════════"
echo "  🎉 Quick Start Complete!"
echo "════════════════════════════════════════════════════════════"
echo
echo "Great job! You've experienced the core of petalTongue."
echo
echo "What you've learned:"
echo "  ✓ Basic graph visualization"
echo "  ✓ Multiple layout algorithms"
echo "  ✓ Interactive zoom, pan, selection"
echo
echo "Continue your journey:"
echo "  • Audio sonification: cd 01-local-primal/03-audio-sonification/"
echo "  • Animation system: cd 01-local-primal/04-animation-flow/"
echo "  • Full Phase 1: ./RUN_ALL_LOCAL.sh"
echo "  • Explore all: See 00_SHOWCASE_INDEX.md"
echo
echo "Time spent: ~25 minutes"
echo "Time well invested! 🌸"
echo

