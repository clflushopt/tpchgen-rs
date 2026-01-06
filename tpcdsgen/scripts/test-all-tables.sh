#!/usr/bin/env bash
#
# Test all ported Rust tables against Java reference fixtures
#
# Usage:
#   ./scripts/test-all-tables.sh [--quiet]
#
# Exit codes:
#   0 - All tables match
#   1 - One or more tables differ

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

# Configuration (can be overridden by --scale)
SCALE_FACTOR=${TPCDS_SCALE:-1}
QUIET=0

# Logging functions
log_info() {
    if [[ $QUIET -eq 0 ]]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    fi
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

# Print usage
usage() {
    cat << EOF
Test all ported Rust tables against Java reference fixtures

Usage:
    $(basename "$0") [--scale N] [--quiet]

Options:
    --scale N       Scale factor (default: 1)
    --quiet         Quiet mode (show only summary)

Examples:
    $(basename "$0")              # Test all tables at scale 1
    $(basename "$0") --scale 10   # Test all tables at scale 10
    $(basename "$0") --quiet      # Test all tables (quiet)

Exit codes:
    0 - All tables match exactly
    1 - One or more tables differ

EOF
    exit 0
}

# All TPC-DS tables to test (24 tables - excludes dbgen_version which has timestamps)
# Note: dbgen_version is excluded because it contains timestamps that will never match
ALL_TABLES=(
    "call_center"
    "catalog_page"
    "catalog_returns"
    "catalog_sales"
    "customer"
    "customer_address"
    "customer_demographics"
    "date_dim"
    "household_demographics"
    "income_band"
    "inventory"
    "item"
    "promotion"
    "reason"
    "ship_mode"
    "store"
    "store_returns"
    "store_sales"
    "time_dim"
    "warehouse"
    "web_page"
    "web_returns"
    "web_sales"
    "web_site"
)

# Get list of tables to test
get_tables_to_test() {
    echo "${ALL_TABLES[@]}"
}

# Build the unified Rust table generator
build_generator() {
    log_info "Building Rust TPC-DS generator..."

    if cargo build --release --quiet 2>&1; then
        log_success "Generator built successfully"
        return 0
    else
        log_error "Failed to build Rust generator"
        return 1
    fi
}

# Test a single table
test_table() {
    local table=$1
    local compare_script="$SCRIPT_DIR/compare-table.sh"

    if [[ $QUIET -eq 1 ]]; then
        "$compare_script" "$table" --scale "$SCALE_FACTOR" --quiet
    else
        "$compare_script" "$table" --scale "$SCALE_FACTOR"
    fi
}

# Main function
main() {
    local passed_tables=()
    local failed_tables=()
    local start_time
    local end_time

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --scale)
                SCALE_FACTOR="$2"
                shift 2
                ;;
            --quiet)
                QUIET=1
                shift
                ;;
            --help)
                usage
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                ;;
        esac
    done

    log_info "========================================="
    log_info "TPC-DS Table Test Suite"
    log_info "Scale Factor: $SCALE_FACTOR"
    log_info "========================================="

    # Get tables to test
    local tables_to_test
    tables_to_test=$(get_tables_to_test)
    local tables_array=($tables_to_test)
    local total_count=${#tables_array[@]}

    log_info "Testing $total_count tables:"
    for table in "${tables_array[@]}"; do
        log_info "  - $table"
    done
    log_info "========================================="

    # Build generator
    cd "$PROJECT_ROOT"
    if ! build_generator; then
        exit 1
    fi
    log_info "========================================="

    # Test each table
    start_time=$(date +%s)

    for table in "${tables_array[@]}"; do
        log_info ""
        log_info "Testing: $table"
        log_info "-----------------------------------------"

        if test_table "$table"; then
            passed_tables+=("$table")
        else
            failed_tables+=("$table")
        fi

        log_info "-----------------------------------------"
    done

    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Print summary
    echo ""
    log_info "========================================="
    log_info "Test Summary"
    log_info "========================================="
    log_info "Total tables tested: $total_count"
    log_success "Passed: ${#passed_tables[@]}"

    if [[ ${#failed_tables[@]} -gt 0 ]]; then
        log_error "Failed: ${#failed_tables[@]}"
        log_error ""
        log_error "Failed tables:"
        for table in "${failed_tables[@]}"; do
            log_error "  ✗ $table"
        done
    fi

    if [[ ${#passed_tables[@]} -gt 0 ]]; then
        echo ""
        log_success "Passed tables:"
        for table in "${passed_tables[@]}"; do
            log_success "  ✓ $table"
        done
    fi

    log_info ""
    log_info "Total time: ${duration}s"
    log_info "========================================="

    # Exit with error if any tables failed
    if [[ ${#failed_tables[@]} -gt 0 ]]; then
        exit 1
    fi

    exit 0
}

main "$@"
