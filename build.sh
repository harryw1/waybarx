#!/bin/bash
set -e

# Build helper for waybarx
# Usage: ./build.sh [debug|release] [sway|hypr]

BUILD_TYPE="${1:-debug}"
FEATURE="${2:-hypr}"

echo "==> Building waybarx ($BUILD_TYPE mode with $FEATURE feature)"

# Step 1: Build web assets
echo "==> Building web assets..."
cd web
npm install
npm run build
cd ..

# Step 2: Verify web-dist exists
if [ ! -d "web-dist" ]; then
    echo "ERROR: web-dist/ directory not found after build"
    exit 1
fi

if [ ! -f "web-dist/index.html" ]; then
    echo "ERROR: web-dist/index.html not found"
    exit 1
fi

echo "==> Web assets built successfully"

# Step 3: Build Rust binary
echo "==> Building Rust binary..."
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release --features "$FEATURE"
    echo "==> Build complete! Binary at: target/release/waybarx"
else
    cargo build --features "$FEATURE"
    echo "==> Build complete! Binary at: target/debug/waybarx"
fi

echo ""
echo "To run:"
if [ "$BUILD_TYPE" = "release" ]; then
    echo "  ./target/release/waybarx"
else
    echo "  ./target/debug/waybarx"
    echo ""
    echo "Or for development with hot reload:"
    echo "  Terminal 1: cd web && npm run dev"
    echo "  Terminal 2: cargo run --features $FEATURE"
fi
