#!/usr/bin/env bash
#
# BingoCube Use Cases Demo
#
# Demonstrates BingoCube use cases across ecoPrimals ecosystem

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

cd "$PROJECT_ROOT"

cat << 'EOF'
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║              🌐 BingoCube Ecosystem Use Cases Demo 🌐                        ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  BingoCube provides a universal visual language across all ecoPrimals:      ║
║                                                                              ║
║  1. BearDog (Identity & Security)                                            ║
║     • Visual identity cards                                                  ║
║     • Progressive trust protocol                                             ║
║     • Challenge-response verification                                        ║
║                                                                              ║
║  2. Songbird (P2P Trust & Discovery)                                         ║
║     • Peer trust stamps                                                      ║
║     • Federation tower trust levels                                          ║
║     • Visual peer recognition                                                ║
║                                                                              ║
║  3. NestGate (Content & Storage)                                             ║
║     • Content fingerprints                                                   ║
║     • Visual git commits                                                     ║
║     • Redundancy visualization                                               ║
║                                                                              ║
║  4. ToadStool (Compute & Verification)                                       ║
║     • Computation proofs                                                     ║
║     • Real-time progress visualization                                       ║
║     • Result verification                                                    ║
║                                                                              ║
║  5. petalTongue (Universal Visualization)                                    ║
║     • Visual rendering                                                       ║
║     • Audio sonification                                                     ║
║     • Animation & flow                                                       ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Try these seeds to see different use cases:                                 ║
║                                                                              ║
║  • "alice_identity"     - BearDog identity card                              ║
║  • "peer_songbird_42"   - Songbird trust stamp                               ║
║  • "commit_7f3a2b1"     - NestGate content fingerprint                       ║
║  • "compute_job_123"    - ToadStool computation proof                        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

EOF

echo "Building demo application..."
cargo build --release --bin bingocube-demo

echo ""
echo "Launching use cases demo..."
echo "(Try different seeds to see various use cases)"
echo "(Press Ctrl+C to exit)"
echo ""

./target/release/bingocube-demo

