#!/bin/bash

# TN5250R GUI Test Runner
# This script runs the automated GUI tests for TN5250R

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're on a system that supports GUI testing
check_system() {
    print_status "Checking system compatibility..."

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Check for X11 or Wayland
        if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
            print_warning "No display server detected. Installing and starting Xvfb for headless testing..."
            if command -v xvfb-run &> /dev/null; then
                XVFB_CMD="xvfb-run -a"
            else
                print_error "Xvfb not found. Please install xvfb: sudo apt-get install xvfb"
                exit 1
            fi
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        print_status "macOS detected - GUI tests should work natively"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        print_status "Windows detected - GUI tests should work natively"
    else
        print_warning "Unknown OS: $OSTYPE - GUI tests may not work"
    fi
}

# Run specific test categories
run_component_tests() {
    print_status "Running component tests..."
    $XVFB_CMD cargo test tests::gui::components -- --nocapture
    print_success "Component tests completed"
}

run_integration_tests() {
    print_status "Running integration tests..."
    $XVFB_CMD cargo test tests::gui::integration -- --nocapture
    print_success "Integration tests completed"
}

run_visual_tests() {
    print_status "Running visual regression tests..."
    $XVFB_CMD cargo test tests::gui::visual -- --nocapture
    print_success "Visual regression tests completed"
}

run_all_tests() {
    print_status "Running all GUI tests..."
    $XVFB_CMD cargo test --test gui -- --nocapture
    print_success "All GUI tests completed"
}

# Update snapshots (for visual regression tests)
update_snapshots() {
    print_status "Updating visual regression snapshots..."
    $XVFB_CMD cargo test tests::gui::visual -- --nocapture --update-snapshots
    print_success "Snapshots updated"
}

# Show usage
usage() {
    echo "TN5250R GUI Test Runner"
    echo ""
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all          Run all GUI tests (default)"
    echo "  components   Run component tests only"
    echo "  integration  Run integration tests only"
    echo "  visual       Run visual regression tests only"
    echo "  update       Update visual regression snapshots"
    echo ""
    echo "Options:"
    echo "  -h, --help   Show this help message"
    echo "  -v, --verbose Enable verbose output"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all tests"
    echo "  $0 components         # Run component tests"
    echo "  $0 update             # Update snapshots"
}

# Parse command line arguments
COMMAND="all"
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        all|components|integration|visual|update)
            COMMAND=$1
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Enable verbose output if requested
if [ "$VERBOSE" = true ]; then
    set -x
fi

# Main execution
main() {
    print_status "TN5250R GUI Test Runner starting..."

    # Check system compatibility
    check_system

    # Build the project first
    print_status "Building project..."
    cargo build
    print_success "Build completed"

    # Run the requested tests
    case $COMMAND in
        all)
            run_all_tests
            ;;
        components)
            run_component_tests
            ;;
        integration)
            run_integration_tests
            ;;
        visual)
            run_visual_tests
            ;;
        update)
            update_snapshots
            ;;
    esac

    print_success "GUI testing completed successfully!"
}

# Run main function
main "$@"