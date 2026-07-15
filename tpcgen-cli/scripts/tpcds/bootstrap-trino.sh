#!/usr/bin/env bash
#
# bootstrap-trino.sh — Set up the Trino TPC-DS Java reference
# implementation used by `--compat trino` conformance testing.
#
# Please see print_usage() below for details.

set -euo pipefail

print_usage() {
    cat << 'EOF'
bootstrap-trino.sh — Set up the Trino TPC-DS Java reference implementation.

What it does:
    1. Checks that Java 11+ and Maven are installed.
    2. Clones the Trino TPC-DS repository into ../tpcds/ (if not present).
    3. Builds the implementation with `mvn clean package -DskipTests`.
    4. Runs a small smoke test to confirm the JAR works.

Usage:
    bootstrap-trino.sh [OPTIONS]

Options:
    --rebuild       Force rebuild even if the JAR already exists.
    --verify        Only verify the existing installation; do not clone/build.
    --help          Show this help message.

Environment variables:
    TPCDS_TRINO_REPO    Git URL for the Trino TPC-DS repo.
    TPCDS_TRINO_REF     Git ref of the repo to build (default: the commit
                        the Rust port was written against).
                        Default: https://github.com/trinodb/tpcds.git

Requirements: Java 11+, Maven, git.

Output:
    Clones to ../tpcds/ (parallel to this repo) and produces
    ../tpcds/target/tpcds-*-jar-with-dependencies.jar.

Examples:
    bootstrap-trino.sh              # Clone and build if needed.
    bootstrap-trino.sh --rebuild    # Force clean rebuild.
    bootstrap-trino.sh --verify     # Just check existing install.

See scripts/README.md for the full conformance-testing workflow.
EOF
    exit 0
}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TPCDS_ROOT="$(cd "$PROJECT_ROOT/.." && pwd)"
TRINO_DIR="$TPCDS_ROOT/tpcds"

# Configuration
TRINO_REPO_URL="${TPCDS_TRINO_REPO:-https://github.com/trinodb/tpcds.git}"
# The commit of trinodb/tpcds the Rust port was written against (also
# referenced from tpcdsgen source comments). Upstream HEAD no longer
# builds on JDK <= 21 and generates different output.
TRINO_REF="${TPCDS_TRINO_REF:-8a02abbba864feedc2afd078c8153d66a95bb2d4}"
FORCE_REBUILD=0
VERIFY_ONLY=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

# Check if Java/Maven are installed
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v java &> /dev/null; then
        log_error "Java is not installed"
        log_error "Please install Java 11+ (e.g., 'brew install openjdk@11' on macOS)"
        return 1
    fi

    if ! command -v mvn &> /dev/null; then
        log_error "Maven is not installed"
        log_error "Please install Maven (e.g., 'brew install maven' on macOS)"
        return 1
    fi

    local java_version
    java_version=$(java -version 2>&1 | head -1 | cut -d'"' -f2 | cut -d'.' -f1)

    log_success "Java version: $(java -version 2>&1 | head -1)"
    log_success "Maven version: $(mvn -version 2>&1 | head -1)"

    if [[ $java_version -lt 11 ]]; then
        log_warn "Java 11+ recommended (found version $java_version)"
    fi

    return 0
}

# Find the built Trino TPC-DS JAR
find_trino_jar() {
    local jar_file
    jar_file=$(find "$TRINO_DIR/target" -name "tpcds-*-jar-with-dependencies.jar" 2>/dev/null | head -1)
    if [[ -z "$jar_file" ]]; then
        return 1
    fi
    echo "$jar_file"
}

# Clone the Trino TPC-DS repository and pin it to the reference commit.
#
# The Rust port matches this exact commit of the Java implementation.
# Upstream master has since moved on (it now targets JDK 25 via airbase
# 390, and its generator output is no longer what our fixtures encode),
# so building an unpinned HEAD produces a reference that is useless for
# conformance testing.
clone_trino_repo() {
    log_info "Cloning Trino TPC-DS repository..."
    log_info "Source: $TRINO_REPO_URL"
    log_info "Ref:    $TRINO_REF"
    log_info "Target: $TRINO_DIR"

    if [[ -d "$TRINO_DIR" ]]; then
        log_warn "Directory already exists: $TRINO_DIR"

        # Check if it's a git repo
        if [[ ! -d "$TRINO_DIR/.git" ]]; then
            log_error "Directory exists but is not a git repository"
            log_error "Please remove $TRINO_DIR and try again"
            return 1
        fi
    else
        if ! git clone "$TRINO_REPO_URL" "$TRINO_DIR"; then
            log_error "Failed to clone Trino TPC-DS repository"
            return 1
        fi
        log_success "Successfully cloned Trino TPC-DS repository"
    fi

    if ! git -C "$TRINO_DIR" checkout --quiet "$TRINO_REF"; then
        log_error "Failed to check out pinned ref $TRINO_REF"
        return 1
    fi
    log_success "Checked out pinned ref $TRINO_REF"
    return 0
}

# Build the Trino TPC-DS JAR
build_trino() {
    log_info "Building Trino TPC-DS implementation..."

    if [[ ! -d "$TRINO_DIR" ]]; then
        log_error "Trino directory does not exist: $TRINO_DIR"
        return 1
    fi

    cd "$TRINO_DIR"

    # Clean build
    log_info "Running: mvn clean package -DskipTests"
    if ! mvn clean package -DskipTests; then
        log_error "Maven build failed"
        cd - >/dev/null
        return 1
    fi

    cd - >/dev/null

    # Verify JAR was created
    local jar_file
    if jar_file=$(find_trino_jar); then
        local jar_size
        jar_size=$(du -h "$jar_file" | cut -f1)
        log_success "Build complete: $jar_file ($jar_size)"
        return 0
    else
        log_error "Build succeeded but JAR file not found"
        return 1
    fi
}

# Smoke-test the built JAR
test_trino() {
    log_info "Testing Trino TPC-DS JAR..."

    local jar_file
    if ! jar_file=$(find_trino_jar); then
        log_error "JAR file not found"
        return 1
    fi

    # Create temp directory for test
    local temp_dir
    temp_dir=$(mktemp -d)

    # Generate a small test table
    log_info "Generating test table (reason) to verify installation..."
    if java -jar "$jar_file" \
        --table reason \
        --scale 1 \
        --directory "$temp_dir" \
        --overwrite \
        > /dev/null 2>&1; then

        # Check output file
        if [[ -f "$temp_dir/reason.dat" ]]; then
            local row_count
            row_count=$(wc -l < "$temp_dir/reason.dat" | tr -d ' ')
            log_success "Test generation successful ($row_count rows)"
            rm -rf "$temp_dir"
            return 0
        else
            log_error "Test generation failed - no output file"
            rm -rf "$temp_dir"
            return 1
        fi
    else
        log_error "Test generation failed"
        rm -rf "$temp_dir"
        return 1
    fi
}

# Verify the installation
verify_installation() {
    log_info "Verifying Trino TPC-DS installation..."

    # Check directory exists
    if [[ ! -d "$TRINO_DIR" ]]; then
        log_error "Trino directory does not exist: $TRINO_DIR"
        return 1
    fi

    # Check JAR exists
    local jar_file
    if ! jar_file=$(find_trino_jar); then
        log_error "JAR file not found in $TRINO_DIR/target/"
        log_error "Run without --verify to build"
        return 1
    fi

    log_success "Found JAR: $jar_file"

    # Test it works
    if ! test_trino; then
        return 1
    fi

    log_success "Trino TPC-DS installation verified"
    return 0
}

# Main function
main() {
    local start_time
    local end_time
    local duration

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --rebuild)
                FORCE_REBUILD=1
                shift
                ;;
            --verify)
                VERIFY_ONLY=1
                shift
                ;;
            --help)
                print_usage
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    log_info "========================================="
    log_info "Trino TPC-DS Bootstrap"
    log_info "========================================="
    log_info "Trino directory: $TRINO_DIR"
    log_info "Repository: $TRINO_REPO_URL"
    log_info "========================================="

    start_time=$(date +%s)

    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi

    # If verify only, just check and exit
    if [[ $VERIFY_ONLY -eq 1 ]]; then
        if verify_installation; then
            exit 0
        else
            exit 1
        fi
    fi

    # Clone repository if needed
    if [[ ! -d "$TRINO_DIR" ]]; then
        if ! clone_trino_repo; then
            exit 1
        fi
    else
        log_success "Trino repository already exists"
    fi

    # Build if needed or forced
    local jar_file
    if [[ $FORCE_REBUILD -eq 1 ]] || ! find_trino_jar >/dev/null 2>&1; then
        if ! build_trino; then
            exit 1
        fi
    else
        log_success "JAR already built: $(find_trino_jar)"
    fi

    # Test the installation
    if ! test_trino; then
        exit 1
    fi

    end_time=$(date +%s)
    duration=$((end_time - start_time))

    echo ""
    log_info "========================================="
    log_info "Bootstrap Complete"
    log_info "========================================="
    log_success "Trino TPC-DS is ready for conformance testing"
    log_info "Time: ${duration}s"
    log_info ""
    log_info "Next steps:"
    log_info "  ./scripts/tpcds/generate-fixtures.sh      # Generate test fixtures"
    log_info "  ./scripts/tpcds/compare-all-tables.sh     # Run conformance tests"
    log_info "========================================="
}

main "$@"
