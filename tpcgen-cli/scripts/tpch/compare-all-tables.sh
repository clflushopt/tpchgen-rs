#!/usr/bin/env bash
#
# compare-all-tables.sh — Run the TPC-H conformance suite, comparing the
# Rust generator's TBL output against the C dbgen reference.
#
# Please see print_usage() below for details.

set -euo pipefail

print_usage() {
    cat << 'EOF'
compare-all-tables.sh — Run the TPC-H conformance suite for one scale factor.

Builds the Rust generator in release mode, generates all 8 TPC-H tables
with `tpcgen-cli tpch tbl` and two refresh function update sets with
`tpcgen-cli tpch updates`, and compares them against the C dbgen
reference (`dbgen -U 2`). Exits non-zero if any file differs.

By default, comparisons only check each table's MD5 against the expected
hash in tests/fixtures/tpch/scale-N/MD5SUMS (which ships with the repo,
generated from C dbgen by generate-fixtures.sh). No docker and no
reference .tbl fixtures are required.

Pass --full to do a byte-for-byte comparison against the full .tbl
fixtures (showing a row-level diff on mismatch). The fixtures must
already exist locally; populate them with generate-fixtures.sh.

USAGE:
    compare-all-tables.sh [--scale N] [--full]

OPTIONS:
    --scale N     Scale factor (default: 1). MD5SUMS must exist for it.
    --full        Byte-for-byte diff against local .tbl fixtures instead
                  of the MD5-only check.
    --help        Show this help.

ENVIRONMENT:
    TPCGEN_CLI_BIN  Prebuilt tpcgen-cli binary to use instead of building.

EXAMPLES:
    compare-all-tables.sh                     # MD5-only, scale 1.
    compare-all-tables.sh --scale 0.01        # MD5-only, scale 0.01.
    compare-all-tables.sh --scale 1 --full    # byte-for-byte, scale 1.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SCALE_FACTOR=1
FULL=0

TABLES=(nation region part supplier partsupp customer orders lineitem)
UPDATE_SETS=2

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; }

while [[ $# -gt 0 ]]; do
    case $1 in
        --scale)
            SCALE_FACTOR="$2"
            shift 2
            ;;
        --full)
            FULL=1
            shift
            ;;
        --help|-h)
            print_usage
            exit 0
            ;;
        *)
            log_error "Unknown argument: $1"
            print_usage
            exit 1
            ;;
    esac
done

FIXTURE_DIR="$PROJECT_ROOT/tests/fixtures/tpch/scale-${SCALE_FACTOR}"

find_generator() {
    local workspace_root
    if workspace_root=$(cd "$PROJECT_ROOT" && cargo locate-project --workspace --message-format=plain 2>/dev/null | xargs dirname); then
        echo "$workspace_root/target/release/tpcgen-cli"
    else
        echo "$PROJECT_ROOT/target/release/tpcgen-cli"
    fi
}

main() {
    if [[ ! -f "$FIXTURE_DIR/MD5SUMS" ]]; then
        log_error "No MD5SUMS for scale ${SCALE_FACTOR} at $FIXTURE_DIR/MD5SUMS"
        log_error "Generate them with: ./scripts/tpch/generate-fixtures.sh --scale ${SCALE_FACTOR}"
        exit 1
    fi

    log_info "========================================="
    log_info "TPC-H conformance, scale ${SCALE_FACTOR} ($([[ $FULL -eq 1 ]] && echo 'byte-for-byte' || echo 'MD5-only'))"
    log_info "========================================="

    local generator
    if [[ -n "${TPCGEN_CLI_BIN:-}" ]]; then
        generator="$TPCGEN_CLI_BIN"
        log_info "Using prebuilt generator: $generator"
    else
        log_info "Building Rust generator..."
        (cd "$PROJECT_ROOT" && cargo build --locked --release -p tpcgen-cli --quiet)
        generator=$(find_generator)
    fi
    if [[ ! -x "$generator" ]]; then
        log_error "Generator not found at $generator"
        exit 1
    fi

    out_dir=""
    out_dir=$(mktemp -d)
    trap 'rm -rf "$out_dir"' EXIT

    log_info "Generating TPC-H data at scale ${SCALE_FACTOR}..."
    "$generator" tpch tbl --scale-factor "$SCALE_FACTOR" --output-dir "$out_dir"

    local files=()
    for table in "${TABLES[@]}"; do
        files+=("${table}.tbl")
    done

    # The update set files listed in MD5SUMS. Fixtures generated before
    # update sets were added have none, so only generate update sets if
    # there is something to compare them against.
    local update_files
    update_files=$(awk '$2 ~ /\.tbl\.u[0-9]+$|^delete\.[0-9]+$/ {print $2}' "$FIXTURE_DIR/MD5SUMS")
    if [[ -z "$update_files" ]]; then
        log_info "No update sets in $FIXTURE_DIR/MD5SUMS, skipping them"
    else
        log_info "Generating ${UPDATE_SETS} update sets at scale ${SCALE_FACTOR}..."
        "$generator" tpch updates --scale-factor "$SCALE_FACTOR" --sets "$UPDATE_SETS" --output-dir "$out_dir"
        local file
        while IFS= read -r file; do
            files+=("$file")
        done <<< "$update_files"
    fi

    local failed=()
    local start_time
    start_time=$(date +%s)

    for file in "${files[@]}"; do
        local rust_file="$out_dir/${file}"
        if [[ ! -f "$rust_file" ]]; then
            log_error "  ✗ ${file}: Rust generator did not produce ${file}"
            failed+=("$file")
            continue
        fi

        if [[ $FULL -eq 1 ]]; then
            local ref_file="$FIXTURE_DIR/${file}"
            if [[ ! -f "$ref_file" ]]; then
                log_error "  ✗ ${file}: missing fixture ${ref_file}"
                log_error "    Populate fixtures with: ./scripts/tpch/generate-fixtures.sh --scale ${SCALE_FACTOR}"
                failed+=("$file")
                continue
            fi
            if diff -q "$rust_file" "$ref_file" > /dev/null; then
                log_success "  ✓ ${file}"
            else
                log_error "  ✗ ${file}: output differs from C dbgen; first differences:"
                diff "$rust_file" "$ref_file" | head -n 10 | sed 's/^/    /' || true
                failed+=("$file")
            fi
        else
            local expected actual
            expected=$(grep " ${file}\$" "$FIXTURE_DIR/MD5SUMS" | awk '{print $1}')
            if [[ -z "$expected" ]]; then
                log_error "  ✗ ${file}: no entry in $FIXTURE_DIR/MD5SUMS"
                failed+=("$file")
                continue
            fi
            actual=$(md5sum "$rust_file" | awk '{print $1}')
            if [[ "$actual" == "$expected" ]]; then
                log_success "  ✓ ${file} ($actual)"
            else
                log_error "  ✗ ${file}: MD5 mismatch (expected ${expected}, got ${actual})"
                failed+=("$file")
            fi
        fi
    done

    local elapsed=$(( $(date +%s) - start_time ))
    log_info "========================================="
    if [[ ${#failed[@]} -eq 0 ]]; then
        log_success "All ${#files[@]} files match C dbgen at scale ${SCALE_FACTOR} (${elapsed}s)"
    else
        log_error "${#failed[@]} file(s) differ at scale ${SCALE_FACTOR}: ${failed[*]}"
        exit 1
    fi
}

main
