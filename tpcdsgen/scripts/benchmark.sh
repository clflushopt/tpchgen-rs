#!/bin/bash
#
# TPC-DS Benchmark Script
# Measures generation time for all tables at scale factors 1, 10, and 100
#
# Usage: ./scripts/benchmark.sh [--no-output] [--json] [--scales "1 10 100"]
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

# Default settings
NO_OUTPUT=""
JSON=""
SCALES="1 10 100"
OUTPUT_DIR=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-output)
            NO_OUTPUT="--no-output"
            shift
            ;;
        --json)
            JSON="--json"
            shift
            ;;
        --scales)
            SCALES="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            echo "TPC-DS Benchmark Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-output       Don't write output files (measure generation speed only)"
            echo "  --json            Output results as JSON"
            echo "  --scales \"1 10\"   Space-separated list of scale factors (default: \"1 10 100\")"
            echo "  --output-dir DIR  Directory for output files (default: temp directory)"
            echo "  -h, --help        Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                         # Run benchmark at scales 1, 10, 100"
            echo "  $0 --no-output             # Measure generation speed without writing files"
            echo "  $0 --scales \"1 10\"         # Only test scales 1 and 10"
            echo "  $0 --json > results.json   # Save results as JSON"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$PROJECT_DIR"

# Build release binary
echo "Building release binary..."
cargo build --release --bin benchmark 2>&1 | grep -v "Compiling\|Finished" || true
echo ""

BENCHMARK_BIN="$PROJECT_DIR/target/release/benchmark"

if [[ ! -x "$BENCHMARK_BIN" ]]; then
    echo "Error: benchmark binary not found at $BENCHMARK_BIN"
    exit 1
fi

# Create temp directory if no output dir specified and not using --no-output
if [[ -z "$OUTPUT_DIR" && -z "$NO_OUTPUT" ]]; then
    OUTPUT_DIR=$(mktemp -d)
    CLEANUP_DIR=true
else
    CLEANUP_DIR=false
fi

# Results file for comparison
RESULTS_FILE="$PROJECT_DIR/benchmark_results_$(date +%Y%m%d_%H%M%S).txt"

echo "=========================================="
echo "TPC-DS Rust Generator Benchmark"
echo "=========================================="
echo "Date: $(date)"
echo "Host: $(hostname)"
echo "CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || cat /proc/cpuinfo 2>/dev/null | grep 'model name' | head -1 | cut -d: -f2 || echo 'Unknown')"
echo "Scales: $SCALES"
echo ""

if [[ -n "$JSON" ]]; then
    echo "["
    first=true
fi

for scale in $SCALES; do
    if [[ -n "$JSON" ]]; then
        if [[ "$first" == "true" ]]; then
            first=false
        else
            echo ","
        fi
    else
        echo ""
        echo "=========================================="
        echo "Scale Factor: $scale"
        echo "=========================================="
    fi

    if [[ -z "$NO_OUTPUT" ]]; then
        SCALE_OUTPUT_DIR="$OUTPUT_DIR/scale_$scale"
        mkdir -p "$SCALE_OUTPUT_DIR"
        "$BENCHMARK_BIN" --scale "$scale" --output-dir "$SCALE_OUTPUT_DIR" $JSON
    else
        "$BENCHMARK_BIN" --scale "$scale" $NO_OUTPUT $JSON
    fi
done

if [[ -n "$JSON" ]]; then
    echo ""
    echo "]"
fi

# Cleanup temp directory
if [[ "$CLEANUP_DIR" == "true" && -d "$OUTPUT_DIR" ]]; then
    rm -rf "$OUTPUT_DIR"
fi

if [[ -z "$JSON" ]]; then
    echo ""
    echo "=========================================="
    echo "Benchmark complete"
    echo "=========================================="
fi
