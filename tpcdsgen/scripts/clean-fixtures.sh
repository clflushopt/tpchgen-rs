#!/usr/bin/env bash
#
# Clean up generated test fixtures
#
# Usage:
#   ./scripts/clean-fixtures.sh [--yes]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

FIXTURE_DIR="$PROJECT_ROOT/tests/fixtures"
SKIP_CONFIRM=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

# Print usage
usage() {
    cat << EOF
Clean up generated test fixtures

Usage:
    $(basename "$0") [--yes]

Options:
    --yes           Skip confirmation prompt

Examples:
    $(basename "$0")        # Clean with confirmation
    $(basename "$0") --yes  # Clean without confirmation

EOF
    exit 0
}

# Main function
main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --yes)
                SKIP_CONFIRM=1
                shift
                ;;
            --help)
                usage
                ;;
            *)
                log_warn "Unknown option: $1"
                usage
                ;;
        esac
    done

    log_info "========================================="
    log_info "Clean Test Fixtures"
    log_info "========================================="

    # Check if fixture directory exists
    if [[ ! -d "$FIXTURE_DIR" ]]; then
        log_info "No fixtures directory found: $FIXTURE_DIR"
        log_info "Nothing to clean"
        exit 0
    fi

    # Count files
    local file_count
    file_count=$(find "$FIXTURE_DIR" -type f -name "*.dat" | wc -l | tr -d ' ')

    if [[ $file_count -eq 0 ]]; then
        log_info "No fixture files found"
        log_info "Nothing to clean"
        exit 0
    fi

    # Get total size
    local total_size
    total_size=$(du -sh "$FIXTURE_DIR" 2>/dev/null | cut -f1 || echo "unknown")

    log_info "Fixture directory: $FIXTURE_DIR"
    log_info "Files to delete: $file_count .dat files"
    log_info "Total size: $total_size"
    log_info ""

    # Confirm deletion
    if [[ $SKIP_CONFIRM -eq 0 ]]; then
        log_warn "This will delete all generated fixtures!"
        read -p "Are you sure? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Cancelled"
            exit 0
        fi
    fi

    # Delete fixtures
    log_info "Deleting fixtures..."
    rm -rf "$FIXTURE_DIR"

    log_success "Fixtures cleaned successfully"
    log_info "========================================="
}

main "$@"
