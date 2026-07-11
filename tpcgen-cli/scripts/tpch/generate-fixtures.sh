#!/usr/bin/env bash
#
# generate-fixtures.sh — Generate TPC-H reference fixtures with the C
# `dbgen` implementation and record their MD5 checksums.
#
# Please see print_usage() below for details.

set -euo pipefail

print_usage() {
    cat << 'EOF'
generate-fixtures.sh — Generate TPC-H reference fixtures using C dbgen.

Runs the official C `dbgen` (via the ghcr.io/scalytics/tpch-docker image,
using docker or podman) at the requested scale factor and writes the
resulting *.tbl files into tests/fixtures/tpch/scale-N/ together with an
MD5SUMS file.

The MD5SUMS file is checked into git and is what CI compares the Rust
generator's output against (see compare-all-tables.sh). The *.tbl files
themselves are gitignored; they are only needed for byte-for-byte
comparisons with compare-all-tables.sh --full.

USAGE:
    generate-fixtures.sh [--scale N] [--force]

OPTIONS:
    --scale N     Scale factor (default: 1). dbgen accepts fractional
                  scale factors such as 0.001, 0.01 and 0.1.
    --force       Regenerate fixtures even if they already exist.
    --help        Show this help.

REQUIREMENTS:
    docker or podman (set TPCH_CONTAINER_RUNTIME to force one).

EXAMPLES:
    generate-fixtures.sh                  # scale 1
    generate-fixtures.sh --scale 0.01
    generate-fixtures.sh --scale 1 --force
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SCALE_FACTOR=1
FORCE=0
DBGEN_IMAGE=${TPCH_DBGEN_IMAGE:-ghcr.io/scalytics/tpch-docker:main}

TABLES=(nation region part supplier partsupp customer orders lineitem)

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
        --force)
            FORCE=1
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

container_runtime() {
    if [[ -n "${TPCH_CONTAINER_RUNTIME:-}" ]]; then
        echo "$TPCH_CONTAINER_RUNTIME"
    elif command -v docker > /dev/null 2>&1; then
        echo docker
    elif command -v podman > /dev/null 2>&1; then
        echo podman
    else
        return 1
    fi
}

main() {
    local fixture_dir="$PROJECT_ROOT/tests/fixtures/tpch/scale-${SCALE_FACTOR}"

    if [[ -f "$fixture_dir/MD5SUMS" && $FORCE -eq 0 ]]; then
        local have_all=1
        for table in "${TABLES[@]}"; do
            [[ -f "$fixture_dir/${table}.tbl" ]] || have_all=0
        done
        if [[ $have_all -eq 1 ]]; then
            log_info "Fixtures for scale ${SCALE_FACTOR} already exist in $fixture_dir"
            log_info "Use --force to regenerate."
            exit 0
        fi
    fi

    local runtime
    if ! runtime=$(container_runtime); then
        log_error "Neither docker nor podman found; cannot run C dbgen."
        exit 1
    fi
    log_info "Using container runtime: $runtime"
    log_info "Generating TPC-H reference data at scale ${SCALE_FACTOR} with C dbgen..."

    work_dir=""
    work_dir=$(mktemp -d)
    trap 'rm -rf "$work_dir"' EXIT

    "$runtime" run --rm -v "$work_dir:/data:z" "$DBGEN_IMAGE" -vf -s "$SCALE_FACTOR"

    mkdir -p "$fixture_dir"
    rm -f "$fixture_dir"/*.tbl "$fixture_dir/MD5SUMS"

    for table in "${TABLES[@]}"; do
        if [[ ! -f "$work_dir/${table}.tbl" ]]; then
            log_error "dbgen did not produce ${table}.tbl"
            exit 1
        fi
        cp "$work_dir/${table}.tbl" "$fixture_dir/${table}.tbl"
    done

    (cd "$fixture_dir" && md5sum $(printf '%s.tbl ' "${TABLES[@]}") > MD5SUMS)

    log_success "Fixtures written to $fixture_dir"
    log_info "MD5SUMS:"
    sed 's/^/    /' "$fixture_dir/MD5SUMS"
}

main
