#!/usr/bin/env bash
#
# compare-queries.sh — Compare the queries of `tpcgen-cli tpch queries`
# against the C qgen reference.
#
# Please see print_usage() below for details.

set -euo pipefail

print_usage() {
    cat << 'EOF'
compare-queries.sh — Compare `tpcgen-cli tpch queries` against C qgen.

Generates the validation queries (qgen -d) and query streams with random
parameters (qgen -r <seed> -p <stream>) at several scale factors with both
qgen and the Rust generator, and compares them. Exits non-zero if any
query differs.

qgen writes the row count limit of a query in a database specific way
after the query (`:n` in the templates), while the Rust generator emits a
standard `limit` clause. Before comparing, qgen's `where rownum <= N;`
lines (the ORACLE flavor qgen is built with below) are rewritten to
`limit N;`, and blank lines and comments are ignored. Everything else must
match exactly, except that the Rust generator omits the interval precision
in Q1 (`day` instead of `day (3)`).

The reference is qgen from TPC-H 2.17 or later; earlier versions choose
different Q4 and Q22 parameters. To build it:

    git clone https://github.com/gregrahn/tpch-kit.git
    make -C tpch-kit/dbgen MACHINE=LINUX DATABASE=ORACLE qgen

USAGE:
    compare-queries.sh --qgen-dir DIR [--seeds "S1 S2 ..."]

OPTIONS:
    --qgen-dir DIR  Directory with the qgen binary, dists.dss and queries/.
    --seeds LIST    Seeds to compare (default: "1 42 101 1231235959").
    --help          Show this help.

ENVIRONMENT:
    TPCGEN_CLI_BIN  Prebuilt tpcgen-cli binary to use instead of building.
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

QGEN_DIR=""
SEEDS="1 42 101 1231235959"
SCALE_FACTORS=(1 0.01 100 30000)
STREAMS=(0 1 2 40 41)

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; }

while [[ $# -gt 0 ]]; do
    case $1 in
        --qgen-dir)
            QGEN_DIR="$2"
            shift 2
            ;;
        --seeds)
            SEEDS="$2"
            shift 2
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

if [[ -z "$QGEN_DIR" || ! -x "$QGEN_DIR/qgen" ]]; then
    log_error "qgen not found, pass --qgen-dir"
    print_usage
    exit 1
fi
QGEN_DIR="$(cd "$QGEN_DIR" && pwd)"

# Normalize qgen output: drop comments and blank lines, and turn the
# `where rownum <= N;` row count line into a `limit` clause.
normalize_qgen() {
    awk '
        /^--/ { next }
        /^[[:space:]]*$/ { next }
        # tpchgen drops the interval precision in Q1
        { sub(/ day \(3\)$/, " day") }
        /^where rownum <= -?[0-9]+;$/ {
            n = $4; sub(/;$/, "", n)
            if (n > 0) { sub(/;$/, "", last); last = last "\nlimit " n ";" }
            next
        }
        { if (have) print last; last = $0; have = 1 }
        END { if (have) print last }
    '
}

# Normalize Rust output: drop comments and blank lines.
normalize_rust() {
    grep -v -e '^--' -e '^[[:space:]]*$'
}

main() {
    local generator
    if [[ -n "${TPCGEN_CLI_BIN:-}" ]]; then
        generator="$TPCGEN_CLI_BIN"
    else
        log_info "Building Rust generator..."
        (cd "$PROJECT_ROOT" && cargo build --locked --release -p tpcgen-cli --quiet)
        generator="$(cd "$PROJECT_ROOT" && cargo locate-project --workspace --message-format=plain | xargs dirname)/target/release/tpcgen-cli"
    fi

    work=$(mktemp -d)
    trap 'rm -rf "$work"' EXIT

    local cases=0 failed=0
    compare() {
        local name="$1"; shift
        local qgen_args=("$@")
        cases=$((cases + 1))
        (cd "$QGEN_DIR" && DSS_QUERY=queries DSS_CONFIG=. ./qgen "${qgen_args[@]}") \
            | normalize_qgen > "$work/qgen.sql"
        "$generator" tpch queries "${rust_args[@]}" | normalize_rust > "$work/rust.sql"
        if ! diff -q "$work/qgen.sql" "$work/rust.sql" > /dev/null; then
            log_error "  ✗ ${name}: output differs from qgen; first differences:"
            diff "$work/qgen.sql" "$work/rust.sql" | head -n 10 | sed 's/^/    /' || true
            failed=$((failed + 1))
        fi
    }

    for sf in "${SCALE_FACTORS[@]}"; do
        rust_args=(-s "$sf")
        compare "validation, scale ${sf}" -d -s "$sf"
        for seed in $SEEDS; do
            for stream in "${STREAMS[@]}"; do
                rust_args=(-s "$sf" --seed "$seed" --stream "$stream")
                compare "seed ${seed}, stream ${stream}, scale ${sf}" \
                    -r "$seed" -p "$stream" -s "$sf"
            done
        done
    done

    if [[ $failed -eq 0 ]]; then
        log_success "All ${cases} query sets match qgen"
    else
        log_error "${failed} of ${cases} query sets differ from qgen"
        exit 1
    fi
}

main
