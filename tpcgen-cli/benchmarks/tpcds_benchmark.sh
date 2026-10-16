#!/usr/bin/env bash
#
# time TPC-DS data generation, outputs CSV row per
#
# Generators: tpcgen-cli, dsdgen, the Trino Java port and DuckDB.
#
# Usage: tpcds_benchmark.sh [OPTIONS]
#
# Options:
#   --scale-factors "1 10 100 300"  Scale factors to run (default shown).
#   --tools LIST     Comma separated subset of tpcgen-cli,dsdgen,trino,duckdb
#                    (default: all).
#   --formats LIST   Comma separated subset of tbl,parquet (default: both).
#   --threads N      Threads / processes per generator (default: all cores).
#   --runs N         Repetitions of each configuration (default: 1).
#   --output-dir DIR Scratch directory for generated data
#                    (default: ./tpcds-bench-data).
#   --results FILE   CSV file to append to (default: ./tpcds_benchmark.csv).
#   --help           Show this message.
#
# Environment:
#   TPCGEN_CLI  tpcgen-cli binary                         (default: on PATH)
#   DSDGEN_DIR  directory containing dsdgen and tpcds.idx (TPC-DS Tools v4)
#   TRINO_JAR   tpcds-*-jar-with-dependencies.jar
#               (default: the jar built by scripts/tpcds/bootstrap-trino.sh)
#   DUCKDB      duckdb binary                             (default: on PATH)
#
# What runs, per scale factor and format (N processes):
#   tbl     tpcgen-cli  tpcgen-cli tpcds dat --num-threads N
#           dsdgen      dsdgen -PARALLEL N -CHILD i (i = 1..N)
#           trino       java -jar tpcds.jar --parallelism N
#   parquet tpcgen-cli  tpcgen-cli tpcds parquet --num-threads N
#           duckdb      generate: CALL dsdgen(sf) into a .duckdb file
#                       convert:  EXPORT DATABASE (FORMAT parquet)
#                       total:    sum of the two steps
#
# CSV columns:
#   timestamp,host,format,tool,version,step,scale_factor,threads,run,
#   wall_sec,user_sec,sys_sec,peak_rss_mb,output_mb,status
#
# peak_rss_mb comes from /usr/bin/time; for dsdgen (several processes) it
# is the largest single process, not the sum.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

SCALE_FACTORS="1 10 100 300"
TOOLS="tpcgen-cli,dsdgen,trino,duckdb"
FORMATS="tbl,parquet"
THREADS=""
RUNS=1
OUTPUT_DIR="./tpcds-bench-data"
RESULTS="./tpcds_benchmark.csv"

TPCGEN_CLI="${TPCGEN_CLI:-tpcgen-cli}"
DSDGEN_DIR="${DSDGEN_DIR:-}"
TRINO_JAR="${TRINO_JAR:-}"
DUCKDB="${DUCKDB:-duckdb}"

usage() { sed -n '2,/^$/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit "${1:-0}"; }

while [[ $# -gt 0 ]]; do
    case $1 in
        --scale-factors) SCALE_FACTORS=$2; shift 2 ;;
        --tools) TOOLS=$2; shift 2 ;;
        --formats) FORMATS=$2; shift 2 ;;
        --threads) THREADS=$2; shift 2 ;;
        --runs) RUNS=$2; shift 2 ;;
        --output-dir) OUTPUT_DIR=$2; shift 2 ;;
        --results) RESULTS=$2; shift 2 ;;
        --help|-h) usage ;;
        *) echo "Unknown option: $1" >&2; usage 1 ;;
    esac
done

log() { echo "[$(date +%H:%M:%S)] $*" >&2; }
die() { echo "ERROR: $*" >&2; exit 1; }
has() { [[ ",$1," == *",$2,"* ]]; }

if [[ -z $THREADS ]]; then
    THREADS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
fi
HOST=$(hostname -s 2>/dev/null || hostname)

# ---------------------------------------------------------------------------
# Timing: run a shell command string under /usr/bin/time and set
# WALL USER SYS RSS_MB STATUS.
# ---------------------------------------------------------------------------
TIME_BIN=/usr/bin/time
[[ -x $TIME_BIN ]] || die "$TIME_BIN not found (on Debian/Ubuntu: apt install time)"
if "$TIME_BIN" --version >/dev/null 2>&1; then TIME_FLAVOR=gnu; else TIME_FLAVOR=bsd; fi

time_cmd() {
    local cmd=$1 tfile rss_kb
    tfile=$(mktemp)
    if [[ $TIME_FLAVOR == gnu ]]; then
        if "$TIME_BIN" -f "%e %U %S %M" -o "$tfile" bash -c "$cmd"; then STATUS=ok; else STATUS=fail; fi
        read -r WALL USER SYS rss_kb < <(tail -1 "$tfile")
    else
        if "$TIME_BIN" -l -o "$tfile" bash -c "$cmd"; then STATUS=ok; else STATUS=fail; fi
        read -r WALL _ USER _ SYS _ < <(head -1 "$tfile")
        rss_kb=$(( $(awk '/maximum resident set size/ {print $1}' "$tfile") / 1024 ))
    fi
    RSS_MB=$(( rss_kb / 1024 ))
    rm -f "$tfile"
}

dir_mb() { [[ -e $1 ]] && du -sm "$1" | cut -f1 || echo 0; }
fsum() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.2f", a + b }'; }
imax() { (( $1 > $2 )) && echo "$1" || echo "$2"; }

record() {
    # format tool version step sf run wall user sys rss_mb out_mb status
    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
        "$(date -u +%FT%TZ)" "$HOST" "$1" "$2" "$3" "$4" "$5" "$THREADS" "$6" \
        "$7" "$8" "$9" "${10}" "${11}" "${12}" >> "$RESULTS"
}

# run_step FORMAT TOOL VERSION STEP SF RUN OUT_PATH CMD
# Times CMD, measures OUT_PATH, appends a CSV row.
run_step() {
    local fmt=$1 tool=$2 version=$3 step=$4 sf=$5 run=$6 out=$7 cmd=$8
    log "START $fmt/$tool/$step sf=$sf run=$run threads=$THREADS"
    time_cmd "$cmd"
    OUT_MB=$(dir_mb "$out")
    log "DONE  $fmt/$tool/$step sf=$sf wall=${WALL}s rss=${RSS_MB}MB out=${OUT_MB}MB status=$STATUS"
    record "$fmt" "$tool" "$version" "$step" "$sf" "$run" "$WALL" "$USER" "$SYS" "$RSS_MB" "$OUT_MB" "$STATUS"
}

fresh_dir() { rm -rf "$1"; mkdir -p "$1"; }

# ---------------------------------------------------------------------------
# Tool discovery
# ---------------------------------------------------------------------------
TPCGEN_VERSION=""; DSDGEN_VERSION=""; TRINO_VERSION=""; DUCKDB_VERSION=""

if has "$TOOLS" tpcgen-cli; then
    command -v "$TPCGEN_CLI" >/dev/null || die "tpcgen-cli not found (set TPCGEN_CLI)"
    TPCGEN_VERSION=$("$TPCGEN_CLI" --version | awk '{print $2}')
fi
if has "$TOOLS" dsdgen && has "$FORMATS" tbl; then
    [[ -x ${DSDGEN_DIR:-/nonexistent}/dsdgen && -f $DSDGEN_DIR/tpcds.idx ]] \
        || die "set DSDGEN_DIR to a directory containing a built dsdgen and tpcds.idx"
    DSDGEN_DIR=$(cd "$DSDGEN_DIR" && pwd)
    DSDGEN_VERSION=$("$DSDGEN_DIR/dsdgen" -HELP 2>&1 | grep -o 'Version [0-9.]*' | head -1 | cut -d' ' -f2)
fi
if has "$TOOLS" trino && has "$FORMATS" tbl; then
    if [[ -z $TRINO_JAR ]]; then
        TRINO_JAR=$(find "$PROJECT_ROOT/tpcds/target" -name 'tpcds-*-jar-with-dependencies.jar' 2>/dev/null | head -1)
    fi
    [[ -f $TRINO_JAR ]] || die "Trino jar not found; run scripts/tpcds/bootstrap-trino.sh or set TRINO_JAR"
    command -v java >/dev/null || die "java not found"
    TRINO_VERSION=$(basename "$TRINO_JAR" | sed 's/^tpcds-//; s/-jar-with-dependencies.jar$//')
fi
if has "$TOOLS" duckdb && has "$FORMATS" parquet; then
    command -v "$DUCKDB" >/dev/null || die "duckdb not found (set DUCKDB)"
    DUCKDB_VERSION=$("$DUCKDB" --version | awk '{print $1}')
    "$DUCKDB" -c "INSTALL tpcds;" >/dev/null
fi

mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR=$(cd "$OUTPUT_DIR" && pwd)
if [[ ! -s $RESULTS ]]; then
    echo "timestamp,host,format,tool,version,step,scale_factor,threads,run,wall_sec,user_sec,sys_sec,peak_rss_mb,output_mb,status" > "$RESULTS"
fi

log "host=$HOST threads=$THREADS scale_factors=[$SCALE_FACTORS] tools=$TOOLS formats=$FORMATS runs=$RUNS"
log "versions: tpcgen-cli=$TPCGEN_VERSION dsdgen=$DSDGEN_VERSION trino=$TRINO_VERSION duckdb=$DUCKDB_VERSION"
log "results -> $RESULTS, data -> $OUTPUT_DIR"

# ---------------------------------------------------------------------------
# Benchmarks
# ---------------------------------------------------------------------------
bench_tbl_tpcgen() {
    local sf=$1 run=$2 out=$OUTPUT_DIR/tbl-tpcgen-cli
    fresh_dir "$out"
    run_step tbl tpcgen-cli "$TPCGEN_VERSION" generate "$sf" "$run" "$out" \
        "'$TPCGEN_CLI' tpcds dat --scale-factor $sf --num-threads $THREADS --output-dir '$out' --no-progress"
    rm -rf "$out"
}

bench_tbl_dsdgen() {
    # dsdgen truncates long option strings, so run inside the output dir
    # with a short relative -DIR and a symlinked tpcds.idx.
    local sf=$1 run=$2 out=$OUTPUT_DIR/tbl-dsdgen cmd
    fresh_dir "$out"
    ln -sf "$DSDGEN_DIR/tpcds.idx" "$out/tpcds.idx"
    if (( THREADS > 1 )); then
        cmd="cd '$out' && pids=(); for i in \$(seq 1 $THREADS); do
            '$DSDGEN_DIR/dsdgen' -SCALE $sf -DIR . -PARALLEL $THREADS -CHILD \$i -FORCE Y -QUIET Y & pids+=(\$!); done
            for p in \"\${pids[@]}\"; do wait \"\$p\" || exit 1; done"
    else
        cmd="cd '$out' && '$DSDGEN_DIR/dsdgen' -SCALE $sf -DIR . -FORCE Y -QUIET Y"
    fi
    run_step tbl dsdgen "$DSDGEN_VERSION" generate "$sf" "$run" "$out" "$cmd"
    rm -rf "$out"
}

bench_tbl_trino() {
    local sf=$1 run=$2 out=$OUTPUT_DIR/tbl-trino
    fresh_dir "$out"
    run_step tbl trino "$TRINO_VERSION" generate "$sf" "$run" "$out" \
        "java -jar '$TRINO_JAR' --scale $sf --parallelism $THREADS --directory '$out' --overwrite"
    rm -rf "$out"
}

bench_parquet_tpcgen() {
    local sf=$1 run=$2 out=$OUTPUT_DIR/parquet-tpcgen-cli
    fresh_dir "$out"
    run_step parquet tpcgen-cli "$TPCGEN_VERSION" generate "$sf" "$run" "$out" \
        "'$TPCGEN_CLI' tpcds parquet --scale-factor $sf --num-threads $THREADS --output-dir '$out' --no-progress"
    rm -rf "$out"
}

bench_parquet_duckdb() {
    local sf=$1 run=$2 out=$OUTPUT_DIR/parquet-duckdb db pq
    db=$out/tpcds.duckdb; pq=$out/parquet
    fresh_dir "$out"
    run_step parquet duckdb "$DUCKDB_VERSION" generate "$sf" "$run" "$db" \
        "'$DUCKDB' '$db' -c \"SET threads=$THREADS; LOAD tpcds; CALL dsdgen(sf=$sf);\" >/dev/null"
    local g_wall=$WALL g_user=$USER g_sys=$SYS g_rss=$RSS_MB g_status=$STATUS
    if [[ $g_status == ok ]]; then
        run_step parquet duckdb "$DUCKDB_VERSION" convert "$sf" "$run" "$pq" \
            "'$DUCKDB' '$db' -c \"SET threads=$THREADS; EXPORT DATABASE '$pq' (FORMAT parquet);\" >/dev/null"
    else
        WALL=0; USER=0; SYS=0; RSS_MB=0; STATUS=skipped; OUT_MB=0
        record parquet duckdb "$DUCKDB_VERSION" convert "$sf" "$run" 0 0 0 0 0 skipped
    fi
    [[ $STATUS == ok ]] && STATUS=$g_status
    record parquet duckdb "$DUCKDB_VERSION" total "$sf" "$run" \
        "$(fsum "$g_wall" "$WALL")" "$(fsum "$g_user" "$USER")" "$(fsum "$g_sys" "$SYS")" \
        "$(imax "$g_rss" "$RSS_MB")" "$OUT_MB" "$STATUS"
    rm -rf "$out"
}

for sf in $SCALE_FACTORS; do
    for run in $(seq 1 "$RUNS"); do
        if has "$FORMATS" tbl; then
            has "$TOOLS" tpcgen-cli && bench_tbl_tpcgen "$sf" "$run"
            has "$TOOLS" dsdgen && bench_tbl_dsdgen "$sf" "$run"
            has "$TOOLS" trino && bench_tbl_trino "$sf" "$run"
        fi
        if has "$FORMATS" parquet; then
            has "$TOOLS" tpcgen-cli && bench_parquet_tpcgen "$sf" "$run"
            has "$TOOLS" duckdb && bench_parquet_duckdb "$sf" "$run"
        fi
    done
done

log "finished; results in $RESULTS"
