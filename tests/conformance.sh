#!/bin/bash

# This script runs conformance tests against `dbgen` official implementation
# for scale factors 0.001, 0.01 and 0.1 and 1.

# Bash niceness.
#
# Exit immediately if a command exits with a non-zero status
set -e
# Treat unset variables as an error when substituting
set -u
# Consider the exit status of all commands in a pipeline
set -o pipefail
# Print commands before executing them
set -x

# Scale factors to run against.
SCALE_FACTORS=("0.001" "0.01" "0.1" "1")

# Define tables to compare, we compare all tables but be explicit about them.
TABLES=("nation" "region" "part" "supplier" "partsupp" "customer" "orders" "lineitem")

# Build the Rust generator
echo "Building tpchgen-rs..."
cargo build --release

# Run tests for each scale factor
for SF in "${SCALE_FACTORS[@]}"; do
    echo "Testing scale factor ${SF}..."
    
    # Create output directories
    RUST_DIR="/tmp/tpchgen-rs-${SF}"
    C_DIR="/tmp/tpchgen-c-${SF}"
    
    rm -rf "${RUST_DIR}" "${C_DIR}"
    mkdir -p "${RUST_DIR}" "${C_DIR}"
    
    # Generate data using Rust implementation
    echo "Generating data with Rust implementation at SF=${SF}..."
    cargo run --release --bin tpchgen-cli -- --scale-factor "${SF}" --output-dir "${RUST_DIR}"
    
    # Generate data using C implementation
    echo "Generating data with C implementation at SF=${SF}..."
    docker run -v "${C_DIR}:/data" --rm ghcr.io/scalytics/tpch-docker:main -vf -s "${SF}"
    
    # Compare files
    DIFF_COUNT=0
    DIFF_ERRORS=""
    
    for TABLE in "${TABLES[@]}"; do
        RUST_FILE="${RUST_DIR}/${TABLE}.tbl"
        C_FILE="${C_DIR}/${TABLE}.tbl"
        
        # Ensure both files exist
        if [[ ! -f "${RUST_FILE}" ]]; then
            echo "ERROR: Rust implementation did not generate ${TABLE}.tbl"
            DIFF_COUNT=$((DIFF_COUNT + 1))
            DIFF_ERRORS="${DIFF_ERRORS}\nMissing file: ${RUST_FILE}"
            continue
        fi
        
        if [[ ! -f "${C_FILE}" ]]; then
            echo "ERROR: C implementation did not generate ${TABLE}.tbl"
            DIFF_COUNT=$((DIFF_COUNT + 1))
            DIFF_ERRORS="${DIFF_ERRORS}\nMissing file: ${C_FILE}"
            continue
        fi
        
        # Compare files
        echo "Comparing ${TABLE}.tbl..."
        
        if ! diff -q "${RUST_FILE}" "${C_FILE}" > /dev/null; then
            echo "ERROR: ${TABLE}.tbl files differ!"
            # Get a few sample differences
            DIFF_SAMPLE=$(diff "${RUST_FILE}" "${C_FILE}" | head -n 10)
            DIFF_COUNT=$((DIFF_COUNT + 1))
            DIFF_ERRORS="${DIFF_ERRORS}\nDifferences in ${TABLE}.tbl:\n${DIFF_SAMPLE}\n"
        else
            echo "SUCCESS: ${TABLE}.tbl files match!"
        fi
    done
    
    # Report results
    echo "--------------------------------------------"
    echo "Scale Factor ${SF} Results:"
    if [[ ${DIFF_COUNT} -eq 0 ]]; then
        echo "All tables match! ✅"
    else
        echo "${DIFF_COUNT} tables have differences! ❌"
        echo -e "${DIFF_ERRORS}"
        echo "Test failed for scale factor ${SF}"
        exit 1
    fi
    echo "--------------------------------------------"
done

echo "All conformance tests passed successfully! ✅"
exit 0 