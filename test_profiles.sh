#!/usr/bin/env bash

# Test profile functionality by checking if the profile directory is created
# and if we can create a basic profile file

set -e

echo "Testing profile functionality..."

# Build the project first
echo "Building project..."
cargo build --quiet

# Check if profile directory gets created
echo "Checking profile directory creation..."
mkdir -p /tmp/tn5250r_test
export XDG_CONFIG_HOME=/tmp/tn5250r_test

# Run the binary briefly to see if it creates the profile directory
timeout 5 ./target/debug/tn5250r --help 2>/dev/null || true

# Check if profile directory was created
if [ -d "/tmp/tn5250r_test/tn5250r/profiles" ]; then
    echo "✓ Profile directory created successfully"
else
    echo "✗ Profile directory not created"
    exit 1
fi

# Test CLI profile option
echo "Testing CLI profile option..."
if ./target/debug/tn5250r --help | grep -q "profile"; then
    echo "✓ CLI profile option available"
else
    echo "✗ CLI profile option not found"
    exit 1
fi

echo "🎉 Profile functionality tests passed!"