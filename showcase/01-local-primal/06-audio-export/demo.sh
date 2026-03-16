#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Audio Export
# Duration: ~30 seconds
# Prerequisites: petalTongue built (cargo test accessible)
# External deps: none (aplay optional for playback)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Audio Export (Pure Rust WAV)"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "Run audio module tests to generate WAV"
print_command "cargo test -p petal-tongue-core test_audio --release -- --nocapture"
print_info "Running audio tests that exercise Pure Rust WAV generation (hound crate)..."

TEST_OUT=$(cd "${PROJECT_ROOT}" && cargo test -p petal-tongue-core audio --release 2>&1 || echo "TESTS_FAILED")
if echo "$TEST_OUT" | grep -q "test result.*ok"; then
    record_pass "Audio module tests pass"
else
    if echo "$TEST_OUT" | grep -q "0 passed"; then
        record_skip "No audio tests found in petal-tongue-core"
    else
        record_fail "Audio tests failed"
        print_output "$(echo "$TEST_OUT" | tail -5)"
    fi
fi

step 3 "Verify WAV generation capability"
print_info "petalTongue uses the 'hound' crate for Pure Rust WAV encoding."
print_info "Audio sonification maps graph data to sound:"
print_info "  - Node count   -> number of tones"
print_info "  - Health       -> pitch (healthy = harmonic)"
print_info "  - Position     -> stereo panning"
print_info "  - Status       -> timbre"

WAV_FILE="${DEMO_OUTPUT_DIR}/showcase-test.wav"
if ls "${PROJECT_ROOT}"/target/*/build/*/out/*.wav 2>/dev/null | head -1 > /dev/null 2>&1; then
    record_pass "WAV artifacts found in build output"
elif [[ -f "$WAV_FILE" ]]; then
    record_pass "WAV file: $WAV_FILE"
else
    record_skip "No WAV file generated (tests may not write to disk)"
fi

step 4 "Playback (optional)"
if command -v aplay &> /dev/null; then
    print_info "aplay detected - WAV playback available"
    record_pass "Audio playback tool available"
elif command -v paplay &> /dev/null; then
    print_info "paplay detected - WAV playback available"
    record_pass "Audio playback tool available"
else
    print_info "No audio playback tool found (aplay/paplay). WAV files can be played externally."
    record_skip "No audio playback tool"
fi

echo
print_results || true
demo_complete "07-graph-layouts"
