#!/usr/bin/env bash
#
# Build and capture a commit-tagged CPU profile of data generation.
#
# Produces, under benchmarks/profiles/<bench>-sf<scale>-<commit>/:
#   flamegraph.svg     interactive flamegraph (open in a browser)
#   report.txt         perf report top-function summary
#   firefox.perf.gz    drag-and-drop into https://profiler.firefox.com
#   run.log            generator output (includes wall time)
#   meta.txt           commit, date, host, exact command
#
# Usage: benchmarks/profile.sh [-b tpcds|tpch] [-s SCALE] [-F HZ] [-k]
#   -b  benchmark to profile (default: tpcds)
#   -s  scale factor (default: 1)
#   -F  perf sampling frequency; keep it prime-ish (default: 997)
#   -k  also keep the raw perf.data in the output directory
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
BENCH=tpcds
SCALE=1
FREQ=997
KEEP_PERF_DATA=0

while getopts "b:s:F:kh" opt; do
    case "$opt" in
        b) BENCH="$OPTARG" ;;
        s) SCALE="$OPTARG" ;;
        F) FREQ="$OPTARG" ;;
        k) KEEP_PERF_DATA=1 ;;
        h) sed -n '2,17p' "$0"; exit 0 ;;
        *) exit 1 ;;
    esac
done

# Tag artifacts with the commit they profile; never let a dirty tree
# masquerade as a clean commit. Catches staged and unstaged changes to
# tracked files, plus untracked source files (but not untracked artifacts
# like SVGs or notes, which don't affect the build).
COMMIT="$(git -C "$REPO_ROOT" rev-parse --short HEAD)"
DIRTY=""
if ! git -C "$REPO_ROOT" diff --quiet HEAD 2>/dev/null ||
    [ -n "$(git -C "$REPO_ROOT" status --porcelain --untracked-files=normal -- '*.rs' 'Cargo.*')" ]; then
    DIRTY="-dirty"
fi
COMMIT="${COMMIT}${DIRTY}"

OUT_DIR="$REPO_ROOT/benchmarks/profiles/${BENCH}-sf${SCALE}-${COMMIT}"
DATA_DIR="$(mktemp -d /tmp/tpcgen-profile.XXXXXX)"
PERF_DATA="$DATA_DIR/perf.data"
trap 'rm -rf "$DATA_DIR"' EXIT
mkdir -p "$OUT_DIR"

echo "==> Building tpcgen (release + debug symbols)"
# The workspace release profile strips debuginfo; keep it for symbolication.
CARGO_PROFILE_RELEASE_DEBUG=true CARGO_PROFILE_RELEASE_STRIP=none \
    cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" -p tpcgen --bin tpcgen

GEN_CMD=("$REPO_ROOT/target/release/tpcgen" "$BENCH" -s "$SCALE" \
    --output-dir "$DATA_DIR/data" --no-progress)

echo "==> Recording profile (${FREQ}Hz, dwarf call graphs)"
mkdir -p "$DATA_DIR/data"
perf record -F "$FREQ" --call-graph dwarf,16384 -o "$PERF_DATA" -- \
    "${GEN_CMD[@]}" | tee "$OUT_DIR/run.log"

echo "==> Writing artifacts to ${OUT_DIR#"$REPO_ROOT"/}"
flamegraph --perfdata "$PERF_DATA" -o "$OUT_DIR/flamegraph.svg"
perf report -i "$PERF_DATA" --stdio --no-children -s symbol \
    --percent-limit 0.5 > "$OUT_DIR/report.txt" 2>/dev/null
perf script -F +pid -i "$PERF_DATA" 2>/dev/null | gzip > "$OUT_DIR/firefox.perf.gz"
if [ "$KEEP_PERF_DATA" -eq 1 ]; then
    cp "$PERF_DATA" "$OUT_DIR/perf.data"
fi

{
    echo "commit:  $(git -C "$REPO_ROOT" rev-parse HEAD)${DIRTY}"
    echo "date:    $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "host:    $(uname -srm)"
    echo "command: ${GEN_CMD[*]}"
    echo "freq:    ${FREQ}Hz"
} > "$OUT_DIR/meta.txt"

echo "==> Done:"
ls -lh "$OUT_DIR"
grep -h "Completed in" "$OUT_DIR/run.log" || true
