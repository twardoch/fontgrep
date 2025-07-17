#!/bin/bash
# this_file: scripts/test.sh
# Test script for fontgrep

set -euo pipefail

echo "🧪 Running fontgrep tests..."

# Run unit tests
echo "📋 Running unit tests..."
cargo test --lib --all-features

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration_tests --all-features

# Run unit tests from separate file
echo "🔬 Running additional unit tests..."
cargo test --test unit_tests --all-features

# Run doctests
echo "📚 Running doctests..."
cargo test --doc --all-features

# Run benchmarks (if available)
if [ -f "benches/bench.rs" ]; then
    echo "⚡ Running benchmarks..."
    cargo bench --all-features
fi

# Check code coverage (if tools available)
if command -v cargo-tarpaulin >/dev/null 2>&1; then
    echo "📊 Running code coverage..."
    cargo tarpaulin --all-features --out Html --output-dir coverage
    echo "📈 Coverage report generated in coverage/tarpaulin-report.html"
fi

echo "✅ All tests passed!"