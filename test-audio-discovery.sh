#!/usr/bin/env bash
# Test Audio Discovery - Verify substrate-agnostic audio system
# 
# This script tests the new AudioManager runtime discovery system

set -euo pipefail

echo "🎵 Testing Substrate-Agnostic Audio Discovery"
echo "=============================================="
echo ""

# Check what audio systems are available on this machine
echo "📊 System Audio Information:"
echo ""

# Check for PipeWire
if [ -e /run/user/$(id -u)/pipewire-0 ]; then
    echo "  ✅ PipeWire socket found: /run/user/$(id -u)/pipewire-0"
    PIPEWIRE_AVAILABLE=true
else
    echo "  ⏭️  PipeWire socket not found"
    PIPEWIRE_AVAILABLE=false
fi

# Check for PulseAudio
if [ -e /run/user/$(id -u)/pulse/native ]; then
    echo "  ✅ PulseAudio socket found: /run/user/$(id -u)/pulse/native"
    PULSE_AVAILABLE=true
else
    echo "  ⏭️  PulseAudio socket not found"
    PULSE_AVAILABLE=false
fi

# Check for direct ALSA devices
if [ -d /dev/snd ]; then
    ALSA_COUNT=$(find /dev/snd -name 'pcm*p' 2>/dev/null | wc -l)
    echo "  ✅ ALSA devices found: $ALSA_COUNT device(s) in /dev/snd"
    ALSA_AVAILABLE=true
else
    echo "  ⏭️  ALSA devices not found"
    ALSA_AVAILABLE=false
fi

echo ""
echo "🎯 Expected AudioManager Behavior:"
echo ""

if [ "$PIPEWIRE_AVAILABLE" = true ]; then
    echo "  → Should select: SocketBackend (PipeWire)"
    echo "  → Priority: Tier 2 (highest available)"
elif [ "$PULSE_AVAILABLE" = true ]; then
    echo "  → Should select: SocketBackend (PulseAudio)"
    echo "  → Priority: Tier 2 (highest available)"
elif [ "$ALSA_AVAILABLE" = true ]; then
    echo "  → Should select: DirectBackend (ALSA /dev/snd)"
    echo "  → Priority: Tier 2 (highest available)"
else
    echo "  → Should select: SoftwareBackend or SilentBackend"
    echo "  → Priority: Tier 4-5 (graceful degradation)"
fi

echo ""
echo "🧪 Running AudioManager discovery test..."
echo ""

# Run the audio discovery test
cd /home/eastgate/Development/ecoPrimals/phase2/petalTongue
cargo test --package petal-tongue-ui --lib audio::manager::tests::test_audio_manager_init -- --nocapture 2>&1 | grep -E "(🎵|✅|🔌|🎨|🎼|🔇|Active|Available)"

echo ""
echo "✅ Audio discovery test complete!"
echo ""
echo "📝 Key Verification Points:"
echo "  1. AudioManager initialized without panicking"
echo "  2. Backend discovery completed"
echo "  3. Active backend selected"
echo "  4. All backends enumerated"
echo ""
echo "🌸 Substrate-agnostic audio is working! 🌸"

