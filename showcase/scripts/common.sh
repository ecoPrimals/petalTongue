#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Shared utilities for petalTongue showcase demos
# Source this file in your demo scripts

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Resolve project root (3 levels up from scripts/)
SHOWCASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_ROOT="$(cd "${SHOWCASE_DIR}/.." && pwd)"
SCENARIOS_DIR="${PROJECT_ROOT}/sandbox/scenarios"

# Binary discovery
find_petaltongue_binary() {
    if [[ -f "${PROJECT_ROOT}/target/release/petaltongue" ]]; then
        echo "${PROJECT_ROOT}/target/release/petaltongue"
    elif [[ -f "${PROJECT_ROOT}/target/debug/petaltongue" ]]; then
        echo "${PROJECT_ROOT}/target/debug/petaltongue"
    else
        echo ""
    fi
}

PETALTONGUE_BIN="${PETALTONGUE_BIN:-$(find_petaltongue_binary)}"

# Demo configuration
PAUSE_DURATION=${PAUSE_DURATION:-1}
VERBOSE=${VERBOSE:-false}
DEMO_OUTPUT_DIR="${DEMO_OUTPUT_DIR:-/tmp/petaltongue-showcase}"
mkdir -p "${DEMO_OUTPUT_DIR}"

# --- Print functions ---

print_header() {
    echo
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${CYAN}  $1${NC}"
    echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo
}

print_step() {
    echo -e "  ${BLUE}[$1]${NC} ${GREEN}$2${NC}"
}

print_info() {
    echo -e "  ${DIM}$1${NC}"
}

print_success() {
    echo -e "  ${GREEN}OK${NC}  $1"
}

print_fail() {
    echo -e "  ${RED}FAIL${NC}  $1"
}

print_skip() {
    echo -e "  ${YELLOW}SKIP${NC}  $1"
}

print_warning() {
    echo -e "  ${YELLOW}WARN${NC}  $1"
}

print_output() {
    echo -e "  ${DIM}|${NC} $1"
}

print_command() {
    echo -e "  ${PURPLE}\$${NC} ${BOLD}$1${NC}"
}

# --- Demo flow ---

step() {
    local num=$1
    shift
    echo
    print_step "$num" "$*"
}

pause() {
    if [[ "${PAUSE_DURATION}" != "0" ]]; then
        sleep "${PAUSE_DURATION}"
    fi
}

wait_for_user() {
    echo
    echo -e "  ${DIM}Press Enter to continue...${NC}"
    read -r
}

demo_complete() {
    echo
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "  ${GREEN}Demo complete.${NC}"
    if [[ -n "${1:-}" ]]; then
        echo -e "  ${DIM}Next: $1${NC}"
    fi
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo
}

# --- Prerequisites ---

require_binary() {
    local binary=$1
    if ! command -v "${binary}" &> /dev/null; then
        print_fail "Required binary '${binary}' not found"
        return 1
    fi
}

require_petaltongue() {
    if [[ -z "${PETALTONGUE_BIN}" ]]; then
        print_fail "petalTongue binary not found. Build with: cargo build --release"
        exit 1
    fi
    print_success "petalTongue binary: ${PETALTONGUE_BIN}"
}

require_scenario() {
    local scenario=$1
    local path="${SCENARIOS_DIR}/${scenario}"
    if [[ ! -f "${path}" ]]; then
        print_fail "Scenario not found: ${path}"
        return 1
    fi
    print_success "Scenario: ${scenario}"
}

# --- Port and process management ---

BG_PIDS=()

cleanup_bg() {
    for pid in "${BG_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
    done
    BG_PIDS=()
}

trap cleanup_bg EXIT

check_port_free() {
    local port=$1
    if ss -tlnp 2>/dev/null | grep -q ":${port} "; then
        return 1
    fi
    return 0
}

wait_for_port() {
    local port=$1
    local timeout=${2:-10}
    local elapsed=0
    while ! ss -tlnp 2>/dev/null | grep -q ":${port} "; do
        sleep 0.5
        elapsed=$((elapsed + 1))
        if [[ $elapsed -ge $((timeout * 2)) ]]; then
            print_fail "Port ${port} did not open within ${timeout}s"
            return 1
        fi
    done
    return 0
}

start_petaltongue_bg() {
    local mode=$1
    shift
    local logfile="${DEMO_OUTPUT_DIR}/petaltongue-${mode}-$$.log"
    RUST_LOG="${RUST_LOG:-warn}" "${PETALTONGUE_BIN}" --log-level error "$mode" "$@" \
        > "$logfile" 2>&1 &
    local pid=$!
    BG_PIDS+=("$pid")
    sleep 0.5
    echo "$pid"
}

stop_pid() {
    local pid=$1
    if kill -0 "$pid" 2>/dev/null; then
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
    fi
}

# --- JSON validation ---

check_json_field() {
    local json=$1
    local field=$2
    local expected=$3
    local actual
    actual=$(echo "$json" | python3 -c "import sys,json; print(json.load(sys.stdin).get('$field',''))" 2>/dev/null || echo "")
    if [[ "$actual" == "$expected" ]]; then
        print_success "$field = $expected"
        return 0
    else
        print_fail "$field: expected '$expected', got '$actual'"
        return 1
    fi
}

check_json_valid() {
    local json=$1
    if echo "$json" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# --- Socket utilities ---

find_petaltongue_socket() {
    local runtime_dir="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    local socket=""
    for candidate in \
        "${runtime_dir}/petaltongue"*.sock \
        /tmp/petaltongue*.sock; do
        if [[ -S "$candidate" ]]; then
            socket="$candidate"
            break
        fi
    done
    echo "$socket"
}

check_socket_exists() {
    local socket=$1
    if [[ -S "$socket" ]]; then
        print_success "Socket: $socket"
        return 0
    else
        print_fail "Socket not found: $socket"
        return 1
    fi
}

send_jsonrpc() {
    local socket=$1
    local method=$2
    local params=${3:-null}
    local id=${4:-1}
    local request="{\"jsonrpc\":\"2.0\",\"method\":\"${method}\",\"params\":${params},\"id\":${id}}"
    echo "$request" | socat - UNIX-CONNECT:"${socket}" 2>/dev/null || \
    echo "$request" | nc -U "${socket}" 2>/dev/null || \
    python3 -c "
import socket, json, sys
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect('${socket}')
s.sendall(b'${request}\n')
data = b''
while True:
    chunk = s.recv(4096)
    if not chunk: break
    data += chunk
    if b'\n' in data: break
s.close()
print(data.decode().strip())
" 2>/dev/null || echo '{"error":"failed to connect"}'
}

# --- Discovery probes ---

check_songbird_running() {
    local runtime_dir="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    for candidate in \
        "${runtime_dir}/songbird"*.sock \
        /tmp/songbird*.sock; do
        if [[ -S "$candidate" ]]; then
            print_success "Songbird socket: $candidate"
            return 0
        fi
    done
    return 1
}

check_biomeos_running() {
    local runtime_dir="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    for candidate in \
        "${runtime_dir}/biomeos"*.sock \
        /tmp/biomeos*.sock; do
        if [[ -S "$candidate" ]]; then
            print_success "biomeOS socket: $candidate"
            return 0
        fi
    done
    if curl -sf http://localhost:3000/api/v1/health > /dev/null 2>&1; then
        print_success "biomeOS HTTP: localhost:3000"
        return 0
    fi
    return 1
}

# --- Version info ---

print_version_info() {
    echo -e "  ${DIM}petalTongue Showcase${NC}"
    if [[ -n "${PETALTONGUE_BIN}" ]]; then
        local ver
        ver=$("${PETALTONGUE_BIN}" --version 2>/dev/null || echo "unknown")
        echo -e "  ${DIM}Binary: ${ver}${NC}"
    fi
    echo -e "  ${DIM}Rust: $(rustc --version 2>/dev/null || echo 'not found')${NC}"
    echo
}

# --- Result tracking ---

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

record_pass() {
    PASS_COUNT=$((PASS_COUNT + 1))
    print_success "$1"
}

record_fail() {
    FAIL_COUNT=$((FAIL_COUNT + 1))
    print_fail "$1"
}

record_skip() {
    SKIP_COUNT=$((SKIP_COUNT + 1))
    print_skip "$1"
}

print_results() {
    echo
    echo -e "  ${BOLD}Results:${NC} ${GREEN}${PASS_COUNT} passed${NC}, ${RED}${FAIL_COUNT} failed${NC}, ${YELLOW}${SKIP_COUNT} skipped${NC}"
    if [[ $FAIL_COUNT -gt 0 ]]; then
        return 1
    fi
    return 0
}
