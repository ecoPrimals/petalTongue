#!/usr/bin/env bash
#
# BearDog Identity Verification Demo
#
# Shows how BingoCube enables progressive trust identity verification

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat << 'EOF'
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║              🔐 BearDog Identity Verification with BingoCube 🔐             ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Scenario: Alice proves her identity to Bob through progressive reveals     ║
║                                                                              ║
║  Protocol:                                                                   ║
║  1. Initial Contact (x=0.2) - "I might be Alice"                            ║
║  2. Challenge (x=0.5) - "I am definitely Alice"                             ║
║  3. Full Verification (x=1.0) - "Full identity commitment"                  ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Step 1: INITIAL CONTACT (x=0.2)                                            ║
║                                                                              ║
║  Alice: "I am alice@ecoprimals.bio"                                          ║
║                                                                              ║
║  BingoCube (5/25 cells revealed):                                            ║
║  ┌───────────┐                                                              ║
║  │ · · 🟥 · ·│                                                              ║
║  │ · 🟦 · · ·│                                                              ║
║  │ · · ✱ 🟨 ·│                                                              ║
║  │ 🟩 · · · ·│                                                              ║
║  │ · · · · ·│                                                              ║
║  └───────────┘                                                              ║
║                                                                              ║
║  Bob's Reaction: "Might be Alice... need more proof"                        ║
║  Security: ~2^(-20) forgery probability                                      ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Step 2: CHALLENGE (x=0.5)                                                   ║
║                                                                              ║
║  Bob: "Prove it with challenge 'abc123'"                                     ║
║                                                                              ║
║  BingoCube (13/25 cells revealed):                                           ║
║  ┌───────────┐                                                              ║
║  │ 🟦 · 🟥 🟨 ·│                                                              ║
║  │ · 🟦 🟩 · 🟥│                                                              ║
║  │ 🟨 · ✱ 🟨 🟦│                                                              ║
║  │ 🟩 🟥 · 🟦 ·│                                                              ║
║  │ · 🟨 🟩 · ·│                                                              ║
║  └───────────┘                                                              ║
║                                                                              ║
║  Bob Verifies: Previous 5 cells still present ✅                            ║
║  Bob's Reaction: "This is definitely Alice!"                                 ║
║  Security: ~2^(-50) forgery probability                                      ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  Step 3: FULL VERIFICATION (x=1.0)                                           ║
║                                                                              ║
║  Bob: "Show full identity"                                                   ║
║                                                                              ║
║  BingoCube (25/25 cells revealed):                                           ║
║  ┌───────────┐                                                              ║
║  │ 🟦 🟩 🟥 🟨 🟦│                                                              ║
║  │ 🟨 🟦 🟩 🟨 🟥│                                                              ║
║  │ 🟩 🟥 ✱ 🟦 🟨│                                                              ║
║  │ 🟦 🟩 🟨 🟦 🟩│                                                              ║
║  │ 🟥 🟨 🟩 🟥 🟦│                                                              ║
║  └───────────┘                                                              ║
║                                                                              ║
║  Bob Verifies: All previous cells present ✅                                ║
║  Bob Stores: Pattern for future recognition                                  ║
║  Bob's Reaction: "Trust established!"                                        ║
║  Security: ~2^(-100) forgery probability                                     ║
║                                                                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║  ✅ IDENTITY VERIFIED                                                       ║
║                                                                              ║
║  Why this works:                                                             ║
║  • Progressive trust builds incrementally                                    ║
║  • Each reveal is a subset of the next (nested masks)                       ║
║  • Visual pattern becomes memorable                                          ║
║  • Can't forge without knowing Alice's identity seed                         ║
║                                                                              ║
║  Try it yourself:                                                            ║
║  $ cd ../../local/07-bingocube-visualization                                 ║
║  $ cargo run --release --bin bingocube-demo                                  ║
║  $ # Enter seed: "alice_identity"                                            ║
║  $ # Slide x from 0.2 → 0.5 → 1.0                                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

EOF

echo ""
echo "📖 For more details, see:"
echo "   $SCRIPT_DIR/README.md"
echo ""
echo "🎮 To try interactive demo:"
echo "   cd $SCRIPT_DIR/../../local/07-bingocube-visualization"
echo "   ./demo.sh"
echo ""

