#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Hello petalTongue
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Hello petalTongue"
print_version_info

step 1 "Verify binary exists"
require_petaltongue

step 2 "Run petaltongue status"
print_command "petaltongue status"
OUTPUT=$("${PETALTONGUE_BIN}" --log-level error status 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "petalTongue"; then
    record_pass "status command produces output"
else
    record_fail "status command produced no recognizable output"
fi

step 3 "Verify UniBin compliance"
if echo "$OUTPUT" | grep -qi "unibin"; then
    record_pass "UniBin status reported"
else
    record_fail "UniBin status not found in output"
fi

step 4 "JSON output mode"
print_command "petaltongue status --format json"
JSON_OUTPUT=$("${PETALTONGUE_BIN}" --log-level error status --format json 2>&1)
if check_json_valid "$JSON_OUTPUT"; then
    record_pass "JSON output is valid"
    print_output "$(echo "$JSON_OUTPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d, indent=2)[:300])" 2>/dev/null || echo "$JSON_OUTPUT")"
else
    record_fail "JSON output is invalid"
fi

step 5 "Verify version"
print_command "petaltongue --version"
VERSION=$("${PETALTONGUE_BIN}" --version 2>&1)
echo "  $VERSION"
record_pass "Version: $VERSION"

print_results || true
demo_complete "01-unibin-modes"
