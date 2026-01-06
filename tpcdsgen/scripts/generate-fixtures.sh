#!/usr/bin/env bash
#
# Generate TPC-DS reference fixtures using the Java implementation
#
# Usage:
#   ./scripts/generate-fixtures.sh              # Generate all tables
#   ./scripts/generate-fixtures.sh --quiet      # Generate all tables (quiet mode)
#   ./scripts/generate-fixtures.sh table1 ...   # Generate specific tables
#   ./scripts/generate-fixtures.sh --help       # Show help

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
JAVA_DIR="$PROJECT_ROOT/../tpcds"

# Configuration (can be overridden by --scale)
SCALE_FACTOR=${TPCDS_SCALE:-1}
QUIET=0

# All TPC-DS tables (25 tables)
ALL_TABLES=(
    "call_center"
    "catalog_page"
    "catalog_returns"
    "catalog_sales"
    "customer"
    "customer_address"
    "customer_demographics"
    "date_dim"
    "dbgen_version"
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

# Logging functions
log_info() {
    if [[ $QUIET -eq 0 ]]; then
        echo -e "${BLUE}[INFO]${NC} $*"
    fi
}

log_success() {
    if [[ $QUIET -eq 0 ]]; then
        echo -e "${GREEN}[SUCCESS]${NC} $*"
    fi
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Print usage
usage() {
    cat << EOF
Generate TPC-DS reference fixtures using the Java implementation

Usage:
    $(basename "$0") [OPTIONS] [TABLES...]

Options:
    --scale N       Scale factor (default: 1)
    --quiet         Quiet mode (minimal output)
    --help          Show this help message

Arguments:
    TABLES          Space-separated list of table names to generate
                    If omitted, generates all 25 tables

Examples:
    $(basename "$0")                          # Generate all tables at scale 1
    $(basename "$0") --scale 10               # Generate all tables at scale 10
    $(basename "$0") --quiet                  # Generate all tables (quiet)
    $(basename "$0") call_center warehouse    # Generate specific tables

EOF
    exit 0
}

# Find Java JAR file
find_java_jar() {
    local jar_pattern="$JAVA_DIR/target/tpcds-*-jar-with-dependencies.jar"
    local jar_file

    jar_file=$(ls $jar_pattern 2>/dev/null | head -1)

    if [[ -z "$jar_file" ]]; then
        return 1
    fi

    echo "$jar_file"
    return 0
}

# Build Java implementation if needed
ensure_java_build() {
    log_info "Checking Java implementation..."

    if ! find_java_jar >/dev/null 2>&1; then
        log_warn "Java JAR not found. Building Java implementation..."

        cd "$JAVA_DIR"
        if ! mvn -q clean package -DskipTests; then
            log_error "Failed to build Java implementation"
            exit 1
        fi
        cd - >/dev/null

        log_success "Java implementation built successfully"
    else
        log_info "Java JAR found: $(find_java_jar)"
    fi
}

# Generate a single table
generate_table() {
    local table=$1
    local jar_file
    jar_file=$(find_java_jar)

    log_info "Generating $table..."

    # Create a temporary directory for generation
    local temp_dir
    temp_dir=$(mktemp -d)

    # Generate table in temp directory
    #
    # Run Java generator, filter out DEBUG lines but capture errors
    local output
    if output=$(java -jar "$jar_file" \
        --table "$table" \
        --scale "$SCALE_FACTOR" \
        --overwrite \
        --directory "$temp_dir" \
        2>&1); then

        # Move generated file to fixture directory
        local output_file="$temp_dir/${table}.dat"

        if [[ -f "$output_file" ]]; then
            mv "$output_file" "$FIXTURE_DIR/"

            # Get file info
            local file_size
            local row_count
            file_size=$(du -h "$FIXTURE_DIR/${table}.dat" | cut -f1)
            row_count=$(wc -l < "$FIXTURE_DIR/${table}.dat" | tr -d ' ')

            log_success "$table generated: $row_count rows, $file_size"
        else
            log_error "Expected output file not found: $output_file"
            rm -rf "$temp_dir"
            return 1
        fi
    else
        log_error "Failed to generate $table"
        rm -rf "$temp_dir"
        return 1
    fi

    # Clean up temp directory
    rm -rf "$temp_dir"
    return 0
}

# Main function
main() {
    local tables_to_generate=()
    local start_time
    local end_time
    local success_count=0
    local fail_count=0

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
                tables_to_generate+=("$1")
                shift
                ;;
        esac
    done

    # Set fixture directory based on scale factor
    FIXTURE_DIR="$PROJECT_ROOT/tests/fixtures/scale-$SCALE_FACTOR"

    # If no tables specified, generate all
    if [[ ${#tables_to_generate[@]} -eq 0 ]]; then
        tables_to_generate=("${ALL_TABLES[@]}")
    fi

    log_info "========================================="
    log_info "TPC-DS Fixture Generator"
    log_info "========================================="
    log_info "Scale Factor: $SCALE_FACTOR"
    log_info "Tables to generate: ${#tables_to_generate[@]}"
    log_info "Fixture directory: $FIXTURE_DIR"
    log_info "========================================="

    # Ensure Java build exists
    ensure_java_build

    # Create fixture directory
    mkdir -p "$FIXTURE_DIR"
    log_info "Created fixture directory: $FIXTURE_DIR"

    # Generate tables
    start_time=$(date +%s)

    for table in "${tables_to_generate[@]}"; do
        if generate_table "$table"; then
            ((success_count++)) || true
        else
            ((fail_count++)) || true
        fi
    done

    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Print summary
    echo ""
    log_info "========================================="
    log_info "Generation Complete"
    log_info "========================================="
    log_success "Successfully generated: $success_count tables"

    if [[ $fail_count -gt 0 ]]; then
        log_error "Failed to generate: $fail_count tables"
    fi

    log_info "Total time: ${duration}s"
    log_info "Fixtures saved to: $FIXTURE_DIR"
    log_info "========================================="

    # Exit with error if any tables failed
    if [[ $fail_count -gt 0 ]]; then
        exit 1
    fi
}

main "$@"
