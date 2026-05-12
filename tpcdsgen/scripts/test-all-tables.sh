#!/usr/bin/env bash
#
# test-all-tables.sh — Run the full conformance suite for one compat
# mode, byte-for-byte (MD5 + diff) comparing Rust output against
# reference fixtures. Main entry point used by CI.
#
# Please see print_usage() below for details.

set -euo pipefail

print_usage() {
    cat << 'EOF'
test-all-tables.sh — Run the full conformance suite for one compat mode.

Iterates all 24 TPC-DS tables (dbgen_version is always excluded because
it contains a generation timestamp), builds the Rust generator in release
mode, delegates each per-table comparison to ./scripts/compare-table.sh,
and prints a pass/fail summary. Exits non-zero if any table differs.

Two reference implementations are supported, selected by --compat:
    --compat trino  (default)  Trino TPC-DS Java fixtures in
                               tests/fixtures/scale-N-trino/
                               (generate with
                                ./scripts/generate-fixtures.sh)
    --compat c                 C dsdgen fixtures in
                               tests/fixtures/scale-N-c/
                               (download with
                                ./scripts/generate-fixtures.sh --compat c)

Per-compat skip lists live near the top of the script. As of this
writing, --compat c additionally skips `customer` until
alamb/tpcds-data is regenerated without the iconv ISO-8859-14 -> UTF-8
step that double-encodes non-ASCII country names.

Usage:
    test-all-tables.sh [OPTIONS]

Options:
    --scale N           Scale factor (default: 1).
    --compat trino|c    Reference implementation (default: trino).
    --quiet             Quiet mode (show only summary).
    --help              Show this help message.

Examples:
    test-all-tables.sh                  # All tables at scale 1 vs Trino.
    test-all-tables.sh --scale 10       # All tables at scale 10 vs Trino.
    test-all-tables.sh --compat c       # All tables at scale 1 vs C dsdgen.
    test-all-tables.sh --quiet          # Summary-only output.

Exit codes:
    0 - All tested tables match.
    1 - One or more tables differ.

See scripts/README.md for the full conformance-testing workflow.
EOF
    exit 0
}

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
COMPAT=${TPCDS_COMPAT:-trino}
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

# Tables to skip per compat mode (in addition to dbgen_version, which is
# always skipped because it contains a generation timestamp).
#
# --compat c: customer.dat is skipped because the reference data in
# https://github.com/alamb/tpcds-data was generated through a pipeline that
# accidentally double-UTF-8-encodes the non-ASCII country names (`CÔTE
# D'IVOIRE`, `RÉUNION`). The Rust --compat c output uses raw Latin-1, which
# is what unmodified C dsdgen produces. Once the reference data is
# regenerated without the iconv ISO-8859-14 -> UTF-8 step, this entry can
# be removed.
# TODO(alamb): re-include customer once alamb/tpcds-data has been regenerated.
C_COMPAT_SKIP_TABLES=("customer")

# Get list of tables to test, applying per-compat skip lists.
get_tables_to_test() {
    local skip_list=()
    if [[ "$COMPAT" == "c" ]]; then
        skip_list=("${C_COMPAT_SKIP_TABLES[@]}")
    fi

    local result=()
    for t in "${ALL_TABLES[@]}"; do
        local skip=0
        for s in "${skip_list[@]:-}"; do
            if [[ "$t" == "$s" ]]; then
                skip=1
                break
            fi
        done
        [[ $skip -eq 0 ]] && result+=("$t")
    done
    echo "${result[@]}"
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
        "$compare_script" "$table" --scale "$SCALE_FACTOR" --compat "$COMPAT" --quiet
    else
        "$compare_script" "$table" --scale "$SCALE_FACTOR" --compat "$COMPAT"
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
            --compat)
                COMPAT="$2"
                shift 2
                ;;
            --quiet)
                QUIET=1
                shift
                ;;
            --help)
                print_usage
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                ;;
        esac
    done

    case $COMPAT in
        trino|c) ;;
        *)
            log_error "Unknown --compat value: $COMPAT (expected: trino, c)"
            exit 1
            ;;
    esac

    log_info "========================================="
    log_info "TPC-DS Table Test Suite"
    log_info "Scale Factor: $SCALE_FACTOR"
    log_info "Compat Mode:  $COMPAT"
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
