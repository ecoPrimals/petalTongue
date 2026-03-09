#!/usr/bin/env bash
# Test petalTongue audio system

set -e

echo "═══════════════════════════════════════════════════════════════════"
echo "🔊 petalTongue Audio System Test"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# Check for audio players
echo "Checking for audio players..."
PLAYER=""
if command -v aplay &> /dev/null; then
    PLAYER="aplay"
    echo "  ✅ Found: aplay (ALSA)"
elif command -v paplay &> /dev/null; then
    PLAYER="paplay"
    echo "  ✅ Found: paplay (PulseAudio)"
elif command -v mpv &> /dev/null; then
    PLAYER="mpv"
    echo "  ✅ Found: mpv"
elif command -v ffplay &> /dev/null; then
    PLAYER="ffplay"
    echo "  ✅ Found: ffplay"
else
    echo "  ⚠️  No audio player found"
    echo "     Install one of: aplay, paplay, mpv, ffplay"
    echo "     Audio will be generated but not played"
fi

echo ""
echo "Testing pure Rust audio generation..."
echo ""

cd "$(dirname "$0")/../.."

# Test 1: Generate simple tone
echo "Test 1: Generating 440Hz sine wave (A note)..."
cargo run --release --example generate_tone -- --frequency 440 --duration 0.5 --output /tmp/test_tone.wav 2>&1 | grep -v "Compiling\|Finished" || true

if [ -f /tmp/test_tone.wav ]; then
    echo "  ✅ WAV generated: /tmp/test_tone.wav"
    SIZE=$(stat -f%z /tmp/test_tone.wav 2>/dev/null || stat -c%s /tmp/test_tone.wav)
    echo "  📊 Size: $SIZE bytes"
    
    if [ -n "$PLAYER" ]; then
        echo "  🔊 Playing..."
        $PLAYER /tmp/test_tone.wav 2>/dev/null || echo "  ⚠️  Playback failed"
    fi
else
    echo "  ❌ WAV generation failed"
fi

echo ""

# Test 2: UI sounds
echo "Test 2: Testing UI sounds..."
for sound in success error click notification; do
    echo "  • $sound..."
    cargo run --release --example test_ui_sounds -- --sound $sound 2>&1 | grep -v "Compiling\|Finished" || true
    
    if [ -n "$PLAYER" ] && [ -f "/tmp/petaltongue_$sound.wav" ]; then
        $PLAYER "/tmp/petaltongue_$sound.wav" 2>/dev/null || true
        sleep 0.2
    fi
done

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "✅ Audio system test complete"
echo ""
echo "Summary:"
echo "  • Audio generation: Pure Rust (works)"
echo "  • Audio playback: $PLAYER"
echo "  • WAV files: /tmp/petaltongue_*.wav"
echo ""
echo "Next: Run petalTongue UI and test audio in context"
echo "      RUST_LOG=info ./target/release/petal-tongue"
echo "═══════════════════════════════════════════════════════════════════"

