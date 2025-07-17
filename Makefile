# this_file: Makefile
# Makefile for fontgrep

.PHONY: all build test clean install uninstall fmt clippy doc bench coverage release help

# Default target
all: build

# Build the project
build:
	@echo "🔧 Building fontgrep..."
	cargo build --release

# Run all tests
test:
	@echo "🧪 Running tests..."
	./scripts/test.sh

# Run specific test types
test-unit:
	@echo "📋 Running unit tests..."
	cargo test --lib --all-features

test-integration:
	@echo "🔗 Running integration tests..."
	cargo test --test integration_tests --all-features

test-doc:
	@echo "📚 Running doctests..."
	cargo test --doc --all-features

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf coverage/

# Install locally
install: build
	@echo "📦 Installing fontgrep..."
	cargo install --path .

# Uninstall
uninstall:
	@echo "🗑️  Uninstalling fontgrep..."
	cargo uninstall fontgrep

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

# Check formatting
check-fmt:
	@echo "🔍 Checking code formatting..."
	cargo fmt --all -- --check

# Run clippy
clippy:
	@echo "📎 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
doc:
	@echo "📖 Generating documentation..."
	cargo doc --all-features --no-deps --open

# Run benchmarks
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench --all-features

# Run code coverage
coverage:
	@echo "📊 Running code coverage..."
	cargo tarpaulin --all-features --out Html --output-dir coverage
	@echo "📈 Coverage report generated in coverage/tarpaulin-report.html"

# Security audit
audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Check binary size
size: build
	@echo "📏 Checking binary size..."
	@du -h target/release/fontgrep
	@echo "🔍 Binary composition:"
	@cargo bloat --release --crates

# Quick development check
check: check-fmt clippy test
	@echo "✅ All checks passed!"

# Full build pipeline
ci: check-fmt clippy test build
	@echo "🚀 CI pipeline completed!"

# Release preparation
release:
	@echo "🚀 Preparing release..."
	./scripts/release.sh

# Docker build
docker-build:
	@echo "🐳 Building Docker image..."
	docker build -t fontgrep:latest .

# Docker run
docker-run: docker-build
	@echo "🐳 Running Docker container..."
	docker run --rm -v $(PWD)/testdata:/fonts fontgrep:latest /fonts

# Development setup
setup:
	@echo "🔧 Setting up development environment..."
	rustup component add rustfmt clippy
	cargo install cargo-tarpaulin cargo-audit cargo-bloat

# Run with testdata
run-testdata: build
	@echo "🔍 Running with testdata..."
	./target/release/fontgrep testdata/

# Run with JSON output
run-json: build
	@echo "📄 Running with JSON output..."
	./target/release/fontgrep -j testdata/

# Performance test
perf-test: build
	@echo "⚡ Running performance test..."
	time ./target/release/fontgrep testdata/

# Memory test
mem-test: build
	@echo "🧠 Running memory test..."
	valgrind --tool=memcheck --leak-check=full ./target/release/fontgrep testdata/

# Update dependencies
update:
	@echo "📦 Updating dependencies..."
	cargo update

# Show help
help:
	@echo "fontgrep Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build the project"
	@echo "  test           - Run all tests"
	@echo "  test-unit      - Run unit tests"
	@echo "  test-integration - Run integration tests"
	@echo "  test-doc       - Run doctests"
	@echo "  clean          - Clean build artifacts"
	@echo "  install        - Install locally"
	@echo "  uninstall      - Uninstall"
	@echo "  fmt            - Format code"
	@echo "  check-fmt      - Check formatting"
	@echo "  clippy         - Run clippy"
	@echo "  doc            - Generate documentation"
	@echo "  bench          - Run benchmarks"
	@echo "  coverage       - Run code coverage"
	@echo "  audit          - Run security audit"
	@echo "  size           - Check binary size"
	@echo "  check          - Run quick development checks"
	@echo "  ci             - Run full CI pipeline"
	@echo "  release        - Prepare release"
	@echo "  docker-build   - Build Docker image"
	@echo "  docker-run     - Run Docker container"
	@echo "  setup          - Set up development environment"
	@echo "  run-testdata   - Run with testdata"
	@echo "  run-json       - Run with JSON output"
	@echo "  perf-test      - Run performance test"
	@echo "  mem-test       - Run memory test"
	@echo "  update         - Update dependencies"
	@echo "  help           - Show this help"