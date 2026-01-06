#!/usr/bin/env bash
#
# Bootstrap the Java TPC-DS implementation for conformance testing
#
# This script:
#   1. Clones the Java TPC-DS repository (if needed)
#   2. Builds the Java implementation
#   3. Verifies the build succeeded
#
# Usage:
#   ./scripts/bootstrap-java.sh           # Clone and build
#   ./scripts/bootstrap-java.sh --rebuild # Force rebuild even if exists
#   ./scripts/bootstrap-java.sh --verify  # Just verify, don't clone/build

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TPCDS_ROOT="$(cd "$PROJECT_ROOT/.." && pwd)"
JAVA_DIR="$TPCDS_ROOT/tpcds"

# Configuration
JAVA_REPO_URL="${TPCDS_JAVA_REPO:-https://github.com/trinodb/tpcds.git}"
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

# Print usage
usage() {
    cat << EOF
Bootstrap the Java TPC-DS implementation for conformance testing

Usage:
    $(basename "$0") [OPTIONS]

Options:
    --rebuild       Force rebuild even if JAR exists
    --verify        Only verify installation, don't clone/build
    --help          Show this help message

Environment Variables:
    TPCDS_JAVA_REPO    Git URL for Java TPC-DS repo
                       Default: https://github.com/trinodb/tpcds.git

Examples:
    $(basename "$0")              # Clone and build if needed
    $(basename "$0") --rebuild    # Force clean rebuild
    $(basename "$0") --verify     # Just check if everything works

EOF
    exit 0
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

# Clone the Java repository
clone_java_repo() {
    log_info "Cloning Java TPC-DS repository..."
    log_info "Source: $JAVA_REPO_URL"
    log_info "Target: $JAVA_DIR"

    if [[ -d "$JAVA_DIR" ]]; then
        log_warn "Directory already exists: $JAVA_DIR"

        # Check if it's a git repo
        if [[ -d "$JAVA_DIR/.git" ]]; then
            log_info "Existing git repository found, pulling latest changes..."
            cd "$JAVA_DIR"
            git pull origin master || log_warn "Failed to pull latest changes"
            cd - >/dev/null
            return 0
        else
            log_error "Directory exists but is not a git repository"
            log_error "Please remove $JAVA_DIR and try again"
            return 1
        fi
    fi

    # Clone the repository
    if ! git clone "$JAVA_REPO_URL" "$JAVA_DIR"; then
        log_error "Failed to clone Java repository"
        return 1
    fi

    log_success "Successfully cloned Java TPC-DS repository"
    return 0
}

# Build the Java implementation
build_java() {
    log_info "Building Java TPC-DS implementation..."

    if [[ ! -d "$JAVA_DIR" ]]; then
        log_error "Java directory does not exist: $JAVA_DIR"
        return 1
    fi

    cd "$JAVA_DIR"

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
    if jar_file=$(find_java_jar); then
        local jar_size
        jar_size=$(du -h "$jar_file" | cut -f1)
        log_success "Build complete: $jar_file ($jar_size)"
        return 0
    else
        log_error "Build succeeded but JAR file not found"
        return 1
    fi
}

# Test the Java implementation
test_java() {
    log_info "Testing Java TPC-DS implementation..."

    local jar_file
    if ! jar_file=$(find_java_jar); then
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

# Verify installation
verify_installation() {
    log_info "Verifying Java TPC-DS installation..."

    # Check directory exists
    if [[ ! -d "$JAVA_DIR" ]]; then
        log_error "Java directory does not exist: $JAVA_DIR"
        return 1
    fi

    # Check JAR exists
    local jar_file
    if ! jar_file=$(find_java_jar); then
        log_error "JAR file not found in $JAVA_DIR/target/"
        log_error "Run without --verify to build"
        return 1
    fi

    log_success "Found JAR: $jar_file"

    # Test it works
    if ! test_java; then
        return 1
    fi

    log_success "Java TPC-DS installation verified"
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
                usage
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    log_info "========================================="
    log_info "Java TPC-DS Bootstrap"
    log_info "========================================="
    log_info "Java directory: $JAVA_DIR"
    log_info "Repository: $JAVA_REPO_URL"
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
    if [[ ! -d "$JAVA_DIR" ]]; then
        if ! clone_java_repo; then
            exit 1
        fi
    else
        log_success "Java repository already exists"
    fi

    # Build if needed or forced
    local jar_file
    if [[ $FORCE_REBUILD -eq 1 ]] || ! find_java_jar >/dev/null 2>&1; then
        if ! build_java; then
            exit 1
        fi
    else
        log_success "JAR already built: $(find_java_jar)"
    fi

    # Test the installation
    if ! test_java; then
        exit 1
    fi

    end_time=$(date +%s)
    duration=$((end_time - start_time))

    echo ""
    log_info "========================================="
    log_info "Bootstrap Complete"
    log_info "========================================="
    log_success "Java TPC-DS is ready for conformance testing"
    log_info "Time: ${duration}s"
    log_info ""
    log_info "Next steps:"
    log_info "  ./scripts/generate-fixtures.sh      # Generate test fixtures"
    log_info "  ./scripts/test-all-tables.sh        # Run conformance tests"
    log_info "========================================="
}

main "$@"
