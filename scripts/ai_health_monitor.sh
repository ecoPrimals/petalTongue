#!/bin/bash
# AI-driven petalTongue health monitor
# This script demonstrates how an AI can observe and diagnose petalTongue

set -euo pipefail

STATUS_FILE="${PETALTONGUE_STATUS_FILE:-/tmp/petaltongue_status.json}"

echo "🤖 AI Health Monitor for petalTongue"
echo "══════════════════════════════════════"
echo ""

# Check if petalTongue is running
if [ ! -f "$STATUS_FILE" ]; then
    echo "❌ ERROR: petalTongue status file not found: $STATUS_FILE"
    echo ""
    echo "🔍 Diagnosis: petalTongue is not running or status reporter disabled"
    echo "💡 Suggestion: Start petalTongue with: ./primalBins/petal-tongue"
    exit 1
fi

echo "✅ Found petalTongue status file: $STATUS_FILE"
echo ""

# Parse status
HEALTH=$(jq -r '.health' "$STATUS_FILE" 2>/dev/null || echo "unknown")
TIMESTAMP=$(jq -r '.timestamp' "$STATUS_FILE" 2>/dev/null || echo "unknown")

echo "📊 Overall Health: $HEALTH"
echo "🕐 Last Update: $TIMESTAMP"
echo ""

# Check modalities
echo "🎨 Modality Status:"
echo "────────────────────"

for modality in visual2d audio animation text_description haptic vr3d; do
    available=$(jq -r ".modalities.$modality.available" "$STATUS_FILE" 2>/dev/null || echo "false")
    tested=$(jq -r ".modalities.$modality.tested" "$STATUS_FILE" 2>/dev/null || echo "false")
    reason=$(jq -r ".modalities.$modality.reason" "$STATUS_FILE" 2>/dev/null || echo "unknown")
    
    if [ "$available" = "true" ]; then
        echo "  ✅ $modality: Available"
    elif [ "$tested" = "true" ]; then
        echo "  ❌ $modality: Unavailable - $reason"
    else
        echo "  ⏸️  $modality: Not tested"
    fi
done

echo ""

# Check audio specifically (critical for accessibility)
echo "🔊 Audio System:"
echo "────────────────"

audio_init=$(jq -r '.audio.initialized' "$STATUS_FILE" 2>/dev/null || echo "false")
audio_provider=$(jq -r '.audio.current_provider' "$STATUS_FILE" 2>/dev/null || echo "none")

echo "  Initialized: $audio_init"
echo "  Current Provider: $audio_provider"

# Check last audio event
last_sound=$(jq -r '.audio.last_sound.sound_name' "$STATUS_FILE" 2>/dev/null || echo "none")
if [ "$last_sound" != "none" ] && [ "$last_sound" != "null" ]; then
    sound_success=$(jq -r '.audio.last_sound.success' "$STATUS_FILE")
    sound_time=$(jq -r '.audio.last_sound.timestamp' "$STATUS_FILE")
    
    echo "  Last Sound: '$last_sound' at $sound_time"
    
    if [ "$sound_success" = "true" ]; then
        echo "    ✅ Playback successful"
    else
        error_msg=$(jq -r '.audio.last_sound.error_message' "$STATUS_FILE")
        echo "    ❌ Playback failed: $error_msg"
    fi
fi

# Check for audio failures
failure_count=$(jq '.audio.recent_failures | length' "$STATUS_FILE" 2>/dev/null || echo "0")
if [ "$failure_count" -gt 0 ]; then
    echo ""
    echo "  ⚠️  Recent Failures ($failure_count):"
    jq -r '.audio.recent_failures[]' "$STATUS_FILE" | head -3 | while read -r failure; do
        echo "    - $failure"
    done
fi

echo ""

# Check for issues
echo "🚨 Issues & Warnings:"
echo "──────────────────────"

issue_count=$(jq '.issues | length' "$STATUS_FILE" 2>/dev/null || echo "0")
if [ "$issue_count" -eq 0 ]; then
    echo "  ✅ No issues detected"
else
    echo "  Found $issue_count issue(s):"
    echo ""
    
    jq -c '.issues[]' "$STATUS_FILE" | while read -r issue; do
        severity=$(jq -r '.severity' <<< "$issue")
        category=$(jq -r '.category' <<< "$issue")
        message=$(jq -r '.message' <<< "$issue")
        suggestion=$(jq -r '.suggested_action' <<< "$issue")
        
        case "$severity" in
            critical) icon="🔴" ;;
            error) icon="❌" ;;
            warning) icon="⚠️ " ;;
            *) icon="ℹ️ " ;;
        esac
        
        echo "  $icon [$severity] $category:"
        echo "     $message"
        if [ "$suggestion" != "null" ] && [ -n "$suggestion" ]; then
            echo "     💡 $suggestion"
        fi
        echo ""
    done
fi

# AI Decision Making
echo "🤖 AI Analysis:"
echo "────────────────"

if [ "$HEALTH" = "healthy" ]; then
    echo "  ✅ System is healthy. No action needed."
elif [ "$HEALTH" = "degraded" ]; then
    echo "  ⚠️  System is degraded but functional."
    echo "     AI recommends monitoring and addressing issues."
    
    # Check if audio is the problem
    audio_available=$(jq -r '.modalities.audio.available' "$STATUS_FILE")
    if [ "$audio_available" = "false" ]; then
        echo ""
        echo "  🔊 Audio Issue Detected:"
        echo "     petalTongue is a Universal UI designed for accessibility."
        echo "     Non-functional audio limits accessibility for blind users."
        echo ""
        echo "  💡 AI Suggestion:"
        
        # Check what's missing
        players=$(jq -r '.audio.system_players | length' "$STATUS_FILE")
        if [ "$players" -eq 0 ]; then
            echo "     No audio players found on system."
            echo "     Install: sudo apt-get install mpv"
            echo "     Or: sudo apt-get install alsa-utils"
        else
            echo "     Audio players exist but playback failed."
            echo "     Check audio device configuration."
            echo "     Run: aplay -l"
        fi
    fi
else
    echo "  🔴 System is unhealthy! Immediate attention required."
    echo "     AI cannot provide assistance until petalTongue recovers."
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "🤖 This is AI-first design: The UI tells me what's wrong!"
echo "════════════════════════════════════════════════════════════"

