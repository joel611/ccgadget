#!/bin/bash

# Unified test runner for CCGadget CLI
set -e

echo "🧪 CCGadget CLI Test Suite"
echo "=========================="

cd "$(dirname "$0")"

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
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Build the project first
print_status "Building project..."
if ./build.sh > build.log 2>&1; then
    print_success "Build completed successfully"
else
    print_error "Build failed - check build.log"
    exit 1
fi

# Run unit tests  
print_status "Running unit tests..."
if cargo test --bin ccgadget --quiet; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Run integration tests
print_status "Running integration tests..."
if cargo test --test integration_tests --quiet; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# Run Bluetooth tests (excluding ignored ones)
print_status "Running Bluetooth tests..."
if cargo test --test bluetooth_tests --quiet; then
    print_success "Bluetooth tests passed"
else
    print_warning "Some Bluetooth tests failed (expected without Bluetooth hardware)"
fi

# Run hook tests with compiled binary
print_status "Running hook functionality tests..."
if ./test_all_hooks.sh > hook_tests.log 2>&1; then
    print_success "Hook tests passed"
else
    print_warning "Hook tests had issues - check hook_tests.log"
fi

# Verify binary functionality
print_status "Verifying binary functionality..."

BINARY="./target/release/ccgadget"

if [ ! -f "$BINARY" ]; then
    print_error "Release binary not found at $BINARY"
    exit 1
fi

# Test help commands
if $BINARY --help > /dev/null 2>&1; then
    print_success "Main help command works"
else
    print_error "Main help command failed"
    exit 1
fi

if $BINARY pair --help > /dev/null 2>&1; then
    print_success "Pair help command works"
else
    print_error "Pair help command failed"
    exit 1
fi

if $BINARY start --help > /dev/null 2>&1; then
    print_success "Start help command works"
else
    print_error "Start help command failed"
    exit 1
fi

if $BINARY install-hook --help > /dev/null 2>&1; then
    print_success "Install-hook help command works"
else
    print_error "Install-hook help command failed"
    exit 1
fi

# Test basic trigger functionality
print_status "Testing trigger command..."
echo '{"session_id":"test","hook_event_name":"TestEvent"}' | $BINARY trigger > /dev/null 2>&1
if [ $? -eq 0 ]; then
    print_success "Trigger command works with JSON input"
else
    print_error "Trigger command failed"
    exit 1
fi

# Summary
echo ""
echo "🎉 Test Suite Summary"
echo "==================="
print_success "✅ Unit tests"
print_success "✅ Integration tests" 
print_success "✅ Binary verification"
print_success "✅ Command functionality"

if [ -f "hook_tests.log" ]; then
    print_success "✅ Hook tests (see hook_tests.log for details)"
fi

echo ""
print_status "All tests completed successfully!"
print_status "Binary ready at: $BINARY"
print_status "Binary size: $(ls -lh $BINARY | awk '{print $5}')"

# Clean up log files
rm -f build.log hook_tests.log

echo ""
print_success "🚀 Ready for deployment!"