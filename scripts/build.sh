#!/bin/bash
# this_file: scripts/build.sh
# Build script for fontgrep

set -euo pipefail

echo "🔧 Building fontgrep..."

# Get the current version
VERSION=$(cargo run --bin fontgrep -- --version 2>/dev/null | cut -d' ' -f2 || echo "unknown")
echo "📝 Version: $VERSION"

# Update Cargo.toml version if we have a git tag
if git describe --tags --exact-match HEAD >/dev/null 2>&1; then
    GIT_VERSION=$(git describe --tags --exact-match HEAD | sed 's/^v//')
    echo "📦 Updating Cargo.toml version to $GIT_VERSION"
    sed -i "s/^version = \".*\"/version = \"$GIT_VERSION\"/" Cargo.toml
fi

# Format code
echo "🎨 Formatting code..."
cargo fmt --all

# Check code with clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "🧪 Running tests..."
cargo test --all-features

# Build release binary
echo "🏗️  Building release binary..."
cargo build --release

echo "✅ Build complete!"
echo "📍 Binary location: target/release/fontgrep"

# Show binary info
if [ -f "target/release/fontgrep" ]; then
    echo "📊 Binary size: $(du -h target/release/fontgrep | cut -f1)"
    echo "🔢 Binary version: $(./target/release/fontgrep --version)"
fi