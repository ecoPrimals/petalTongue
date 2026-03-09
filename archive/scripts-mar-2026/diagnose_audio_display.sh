#!/usr/bin/env bash
# petalTongue Audio & Display Diagnostic Tool
# Helps debug issues when using RustDesk with local monitor/audio

set -e

echo "🔍 petalTongue Audio & Display Diagnostic"
echo "========================================"
echo ""

# Check display environment
echo "📺 DISPLAY DETECTION:"
echo "-------------------"
if [ -n "$DISPLAY" ]; then
    echo "✓ DISPLAY=$DISPLAY"
else
    echo "✗ DISPLAY not set"
fi

if [ -n "$WAYLAND_DISPLAY" ]; then
    echo "✓ WAYLAND_DISPLAY=$WAYLAND_DISPLAY"
else
    echo "✗ WAYLAND_DISPLAY not set"
fi

# List all X displays
echo ""
echo "Available X displays:"
ls -la /tmp/.X11-unix/ 2>/dev/null || echo "  (none found)"

# Check for Wayland
echo ""
echo "Wayland sockets:"
ls -la /run/user/$(id -u)/wayland-* 2>/dev/null || echo "  (none found)"

echo ""
echo "🔊 AUDIO DETECTION:"
echo "------------------"

# Check PulseAudio
if command -v pactl &> /dev/null; then
    echo "✓ PulseAudio detected (pactl available)"
    
    echo ""
    echo "Audio sinks (outputs):"
    pactl list sinks short | nl
    
    echo ""
    echo "Default sink:"
    pactl info | grep "Default Sink"
    
    echo ""
    echo "Checking for network audio (RustDesk forwarding):"
    if pactl list sinks | grep -i "network\|tunnel\|tcp\|remote"; then
        echo "⚠️  WARNING: Network audio sink detected!"
        echo "   Audio may be forwarded to RustDesk instead of local speakers"
    else
        echo "✓ No network audio detected (audio should be local)"
    fi
else
    echo "✗ PulseAudio not detected (pactl not found)"
fi

echo ""
# Check ALSA
if [ -f /proc/asound/cards ]; then
    echo "✓ ALSA detected"
    echo "  Sound cards:"
    cat /proc/asound/cards | grep -v "^$" | nl
else
    echo "✗ ALSA not detected"
fi

echo ""
# Check available audio players
echo "🎵 AUDIO PLAYERS:"
echo "----------------"
for player in mpv ffplay paplay aplay vlc; do
    if command -v $player &> /dev/null; then
        echo "✓ $player ($(which $player))"
    else
        echo "✗ $player (not found)"
    fi
done

echo ""
echo "📁 TEMP DIRECTORY:"
echo "-----------------"
echo "Temp dir: $TMPDIR (fallback: /tmp)"
echo "petalTongue signature WAV would be at:"
echo "  ${TMPDIR:-/tmp}/petaltongue_signature.wav"

# Check if signature file exists
if [ -f "${TMPDIR:-/tmp}/petaltongue_signature.wav" ]; then
    echo "✓ Signature WAV found"
    ls -lh "${TMPDIR:-/tmp}/petaltongue_signature.wav"
else
    echo "✗ Signature WAV not found (run petalTongue first)"
fi

echo ""
echo "🎯 RECOMMENDATIONS:"
echo "==================="

# Audio recommendations
if pactl list sinks | grep -qi "network\|tunnel"; then
    echo ""
    echo "AUDIO ISSUE DETECTED:"
    echo "-------------------"
    echo "Your audio is being forwarded through RustDesk."
    echo ""
    echo "To fix (choose one):"
    echo ""
    echo "Option 1: Switch PulseAudio to local speakers"
    echo "  # List sinks"
    echo "  pactl list sinks short"
    echo ""
    echo "  # Set default to local hardware (replace N with local sink number)"
    echo "  pactl set-default-sink N"
    echo ""
    echo "Option 2: Force specific audio device for petalTongue"
    echo "  export PULSE_SINK=alsa_output.pci-0000_00_1f.3.analog-stereo"
    echo "  ./petal-tongue"
    echo ""
    echo "Option 3: Use ALSA directly (bypass PulseAudio)"
    echo "  export PETALTONGUE_AUDIO_BACKEND=alsa"
    echo "  ./petal-tongue"
fi

# Display recommendations
echo ""
echo "DISPLAY RECOMMENDATIONS:"
echo "-----------------------"
echo "If GUI appears on wrong display:"
echo ""
echo "Option 1: Disconnect RustDesk before running"
echo "  # Run petalTongue directly at physical console"
echo "  ./petal-tongue"
echo ""
echo "Option 2: Force specific X display"
echo "  export DISPLAY=:0  # or :1, :2, etc"
echo "  ./petal-tongue"
echo ""
echo "Option 3: Test audio manually first"
echo "  # Test with aplay (ALSA)"
echo "  speaker-test -t wav -c 2"
echo ""
echo "  # Test with paplay (PulseAudio)"
echo "  paplay /usr/share/sounds/alsa/Front_Center.wav"
echo ""
echo "  # Test with mpv"
echo "  mpv --audio-device=help  # List devices"
echo "  mpv --audio-device=alsa/default /path/to/test.mp3"

echo ""
echo "🧪 TEST AUDIO NOW:"
echo "==================="
echo "Run this to test if audio works:"
echo ""
echo "  speaker-test -t wav -c 2 -l 1"
echo ""
echo "If you hear sound, audio is working!"
echo "If not, check the recommendations above."
echo ""

# Check if petalTongue is currently running
echo ""
echo "📊 PETALTONGUE STATUS:"
echo "---------------------"
if pgrep -f "petal-tongue" > /dev/null; then
    echo "✓ petalTongue is currently running"
    echo "  PIDs: $(pgrep -f "petal-tongue" | tr '\n' ' ')"
else
    echo "✗ petalTongue is not running"
fi

echo ""
echo "✅ Diagnostic complete!"
echo ""
echo "Next steps:"
echo "1. Try the audio test above"
echo "2. Apply recommendations if needed"
echo "3. Run: RUST_LOG=debug ./petal-tongue 2>&1 | grep -i audio"
echo "4. Check logs for audio player success/failure"

