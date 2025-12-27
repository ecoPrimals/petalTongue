#!/usr/bin/env bash
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
NC='\033[0m' # No Color

# Demo configuration
PAUSE_DURATION=${PAUSE_DURATION:-2}
VERBOSE=${VERBOSE:-false}

# Print functions
print_header() {
    echo -e "${PURPLE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${PURPLE}║${NC}  ${CYAN}$1${NC}"
    echo -e "${PURPLE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo
}

print_step() {
    echo -e "${BLUE}▶${NC} ${GREEN}Step $1:${NC} $2"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_command() {
    echo -e "${PURPLE}$${NC} ${YELLOW}$1${NC}"
}

# Demo flow functions
step() {
    local num=$1
    shift
    echo
    print_step "$num" "$*"
    echo
}

pause() {
    if [[ "${PAUSE_DURATION}" != "0" ]]; then
        sleep "${PAUSE_DURATION}"
    fi
}

run_command() {
    print_command "$*"
    if [[ "${VERBOSE}" == "true" ]]; then
        "$@"
    else
        "$@" >/dev/null 2>&1
    fi
    pause
}

# Prerequisites checking
check_binary() {
    local binary=$1
    if ! command -v "${binary}" &> /dev/null; then
        print_error "Required binary '${binary}' not found"
        return 1
    fi
    print_success "Found ${binary}"
}

check_petaltongue_built() {
    local target_dir="../../target/release"
    if [[ ! -f "${target_dir}/petal-tongue" ]] && [[ ! -f "${target_dir}/petaltongue" ]]; then
        print_error "petalTongue not built. Run: cargo build --release"
        return 1
    fi
    print_success "petalTongue is built"
}

check_biomeos_running() {
    if ! curl -s http://localhost:3000/health &> /dev/null; then
        print_warning "BiomeOS not running on localhost:3000"
        return 1
    fi
    print_success "BiomeOS is running"
}

check_prerequisites() {
    print_info "Checking prerequisites..."
    echo
    
    check_binary "cargo" || return 1
    check_binary "rustc" || return 1
    
    if [[ "${REQUIRE_PETALTONGUE:-true}" == "true" ]]; then
        check_petaltongue_built || return 1
    fi
    
    if [[ "${REQUIRE_BIOMEOS:-false}" == "true" ]]; then
        check_biomeos_running || return 1
    fi
    
    echo
}

# Cleanup functions
cleanup() {
    print_info "Cleaning up..."
    # Override this function in your demo script if needed
}

trap cleanup EXIT

# Wait for user input
wait_for_user() {
    echo
    print_info "Press Enter to continue..."
    read -r
}

# Show expected output
show_expected_output() {
    echo
    print_info "Expected output:"
    echo -e "${CYAN}$1${NC}"
    echo
}

# Demo completion
demo_complete() {
    echo
    print_header "Demo Complete!"
    print_success "You've successfully completed this demonstration"
    if [[ -n "${1:-}" ]]; then
        echo
        print_info "Next: $1"
    fi
    echo
}

# Version info
print_version_info() {
    print_info "petalTongue Showcase Demo"
    print_info "Version: $(git describe --tags 2>/dev/null || echo 'development')"
    print_info "Rust: $(rustc --version)"
    echo
}

