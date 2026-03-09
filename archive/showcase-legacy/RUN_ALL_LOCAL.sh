#!/usr/bin/env bash
# Run all Phase 1 (Local Primal) demos in sequence
# Total time: ~75 minutes

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="${SCRIPT_DIR}"

clear
echo "════════════════════════════════════════════════════════════"
echo "  🌸 petalTongue Showcase - Phase 1: Local Primal"
echo "════════════════════════════════════════════════════════════"
echo
echo "This will run all 9 Phase 1 demos in sequence."
echo "Estimated total time: ~75 minutes"
echo
echo "You can exit any demo with Ctrl+C and it will move to the next."
echo
read -p "Press Enter to begin, or Ctrl+C to cancel..."
echo

# Array of demos in order
demos=(
    "01-local-primal/00-hello-petaltongue"
    "01-local-primal/01-graph-engine"
    "01-local-primal/02-visual-2d"
    "01-local-primal/03-audio-sonification"
    "01-local-primal/04-animation-flow"
    "01-local-primal/05-dual-modality"
    "01-local-primal/06-capability-detection"
    "01-local-primal/07-audio-export"
    "01-local-primal/08-tool-integration"
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
    
    demo_path="${SHOWCASE_ROOT}/${demo}"
    
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
    
    # Run the demo
    if cd "$demo_path" && bash demo.sh; then
        echo "✅ Demo completed successfully"
    else
        exit_code=$?
        if [[ $exit_code -eq 130 ]]; then
            # Ctrl+C (SIGINT)
            echo "⏭️  Demo interrupted, moving to next..."
        else
            echo "❌ Demo failed with exit code: $exit_code"
            read -p "Continue with remaining demos? [Y/n] " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]] && [[ ! -z $REPLY ]]; then
                echo "Exiting..."
                exit 1
            fi
        fi
    fi
    
    # Pause between demos
    if [[ $current -lt $total ]]; then
        echo
        echo "─────────────────────────────────────────────────────────────"
        read -p "Press Enter for next demo, or Ctrl+C to stop..."
        clear
    fi
done

echo
echo "════════════════════════════════════════════════════════════"
echo "  🎉 Phase 1 Complete!"
echo "════════════════════════════════════════════════════════════"
echo
echo "You've completed all 9 local primal demos!"
echo
echo "What you've mastered:"
echo "  ✓ Basic visualization"
echo "  ✓ Graph engine & layouts"
echo "  ✓ Interactive visual controls"
echo "  ✓ Audio sonification"
echo "  ✓ Animation system"
echo "  ✓ Dual-modality design"
echo "  ✓ Capability detection"
echo "  ✓ Audio export"
echo "  ✓ Tool integration"
echo
echo "Total time: ~75 minutes"
echo
echo "Next: Phase 2 - BiomeOS Integration"
echo "  cd ../02-biomeos-integration/"
echo "  cat README.md"
echo

