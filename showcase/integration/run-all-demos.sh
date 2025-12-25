#!/usr/bin/env bash
#
# Run all BingoCube integration demos sequentially
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat << 'EOF'
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║          🌐 BingoCube Cross-Primal Integration Demo Suite 🌐                ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  This demo suite shows how BingoCube provides a universal visual language   ║
║  across all ecoPrimals primals.                                              ║
║                                                                              ║
║  Demos (5 total):                                                            ║
║  1. BearDog - Identity Verification                                          ║
║  2. Songbird - P2P Trust Stamps                                              ║
║  3. NestGate - Content Fingerprints                                          ║
║  4. ToadStool - Computation Proofs                                           ║
║  5. Cross-Primal - Complete Workflow                                         ║
║                                                                              ║
║  Duration: ~30-45 minutes total                                              ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

EOF

echo ""
read -p "Press Enter to start demo 1/5 (BearDog Identity)..."
bash "$SCRIPT_DIR/01-beardog-identity/demo.sh"

echo ""
read -p "Press Enter to start demo 2/5 (Songbird Trust)..."
bash "$SCRIPT_DIR/02-songbird-trust/demo.sh"

echo ""
read -p "Press Enter to start demo 3/5 (NestGate Content)..."
bash "$SCRIPT_DIR/03-nestgate-content/demo.sh"

echo ""
read -p "Press Enter to start demo 4/5 (ToadStool Compute)..."
bash "$SCRIPT_DIR/04-toadstool-compute/demo.sh"

echo ""
read -p "Press Enter to start demo 5/5 (Cross-Primal Workflow)..."
bash "$SCRIPT_DIR/05-cross-primal/demo.sh"

cat << 'EOF'

╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║                  ✅ ALL DEMOS COMPLETE! ✅                                  ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  You've seen how BingoCube provides:                                         ║
║                                                                              ║
║  ✅ Universal visual language across all primals                            ║
║  ✅ Human-verifiable cryptography                                           ║
║  ✅ Progressive trust building                                              ║
║  ✅ Visual provenance tracking                                              ║
║  ✅ Cross-primal integration patterns                                       ║
║                                                                              ║
║  Next steps:                                                                 ║
║  • Try interactive demo: showcase/local/07-bingocube-visualization/demo.sh  ║
║  • Read whitepaper: whitePaper/BingoCube-Overview.md                         ║
║  • Explore code: crates/bingocube-core/                                      ║
║                                                                              ║
║  Thank you for exploring BingoCube! 🌸                                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

EOF

