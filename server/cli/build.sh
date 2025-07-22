#!/bin/bash

# Build script for CCGadget CLI
set -e

echo "🔨 Building CCGadget CLI..."
echo "=========================="

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build release version
echo "⚙️ Building optimized release binary..."
cargo build --release

# Verify build success
if [ -f "target/release/ccgadget" ]; then
    echo "✅ Build successful!"
    echo "📦 Binary location: $(pwd)/target/release/ccgadget"
    echo "📏 Binary size: $(ls -lh target/release/ccgadget | awk '{print $5}')"
    
    # Test basic functionality
    echo "🧪 Testing basic functionality..."
    ./target/release/ccgadget --help > /dev/null
    echo "✅ Binary is functional"
    
    echo ""
    echo "🎉 Release build complete!"
    echo "To install globally: cargo install --path ."
    echo "To run: ./target/release/ccgadget"
else
    echo "❌ Build failed!"
    exit 1
fi