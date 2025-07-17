# GitHub Actions Workflows Setup

Due to GitHub App permissions, the workflow files need to be created manually in the repository. Here are the recommended workflows:

## 1. CI Workflow (`.github/workflows/ci.yml`)

This workflow runs on every push and pull request to test the code:

```yaml
name: CI

on:
  push:
    branches: ["main"]
  pull_request:
    branches: ["main"]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        exclude:
          - os: macos-latest
            rust: beta
          - os: windows-latest
            rust: beta

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Check clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Build release binary
        run: cargo build --release

      - name: Test release binary
        run: ./target/release/fontgrep --version

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate code coverage
        run: cargo tarpaulin --all-features --workspace --timeout 120 --out xml

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          file: cobertura.xml
          fail_ci_if_error: false
```

## 2. Release Workflow (`.github/workflows/release.yml`)

This workflow triggers on git tags to create releases:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  test:
    name: Test Before Release
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Check clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features --verbose

  build:
    name: Build Release Binaries
    needs: test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: fontgrep-linux-x86_64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            name: fontgrep-linux-x86_64-musl
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            name: fontgrep-windows-x86_64.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            name: fontgrep-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: fontgrep-macos-aarch64

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install musl tools
        if: matrix.target == 'x86_64-unknown-linux-musl'
        run: |
          sudo apt-get update
          sudo apt-get install -y musl-tools

      - name: Build release binary
        run: |
          cargo build --release --target ${{ matrix.target }}

      - name: Strip binary (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          strip target/${{ matrix.target }}/release/fontgrep

      - name: Create tarball (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czf ../../../${{ matrix.name }}.tar.gz fontgrep
          cd ../../..

      - name: Create zip (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../${{ matrix.name }}.zip fontgrep.exe
          cd ../../..

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.name }}
          path: |
            ${{ matrix.name }}.tar.gz
            ${{ matrix.name }}.zip

  release:
    name: Create Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Download artifacts
        uses: actions/download-artifact@v3
        with:
          path: artifacts

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/**/*
          body: |
            ## Release ${{ github.ref_name }}
            
            ### Changes
            See [CHANGELOG.md](https://github.com/${{ github.repository }}/blob/main/CHANGELOG.md) for detailed changes.
            
            ### Installation
            
            #### Binary Downloads
            Download the appropriate binary for your platform:
            
            - **Linux x86_64**: `fontgrep-linux-x86_64.tar.gz`
            - **Linux x86_64 (musl)**: `fontgrep-linux-x86_64-musl.tar.gz`  
            - **Windows x86_64**: `fontgrep-windows-x86_64.exe.zip`
            - **macOS x86_64**: `fontgrep-macos-x86_64.tar.gz`
            - **macOS ARM64**: `fontgrep-macos-aarch64.tar.gz`
            
            #### From Source
            ```bash
            cargo install fontgrep
            ```
            
            #### From Git
            ```bash
            cargo install --git https://github.com/${{ github.repository }} --tag ${{ github.ref_name }}
            ```
            
            ### Usage
            ```bash
            fontgrep --help
            ```
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish:
    name: Publish to crates.io
    needs: test
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Update Cargo.toml version
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CRATES_TOKEN }}
        env:
          CRATES_TOKEN: ${{ secrets.CRATES_TOKEN }}
```

## 3. Pull Request Workflow (`.github/workflows/pr.yml`)

This workflow runs additional checks on pull requests:

```yaml
name: Pull Request

on:
  pull_request:
    branches: ["main"]

env:
  CARGO_TERM_COLOR: always

jobs:
  changes:
    name: Check Changes
    runs-on: ubuntu-latest
    outputs:
      rust: ${{ steps.changes.outputs.rust }}
      docs: ${{ steps.changes.outputs.docs }}
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Check for changes
        uses: dorny/paths-filter@v2
        id: changes
        with:
          filters: |
            rust:
              - 'src/**'
              - 'tests/**'
              - 'benches/**'
              - 'Cargo.toml'
              - 'Cargo.lock'
            docs:
              - 'README.md'
              - 'CHANGELOG.md'
              - 'docs/**'

  test:
    name: Test Suite
    needs: changes
    if: ${{ needs.changes.outputs.rust == 'true' }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Check clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Build release binary
        run: cargo build --release

      - name: Test release binary
        run: ./target/release/fontgrep --version

  benchmark:
    name: Performance Benchmarks
    needs: changes
    if: ${{ needs.changes.outputs.rust == 'true' }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run benchmarks
        run: cargo bench --all-features

      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/

  security:
    name: Security Audit
    needs: changes
    if: ${{ needs.changes.outputs.rust == 'true' }}
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit
```

## Setup Instructions

1. **Create the workflow files manually** in your repository by copying the YAML content above into the appropriate files in `.github/workflows/`

2. **Set up repository secrets** (if needed):
   - `CRATES_TOKEN`: Token for publishing to crates.io (optional)

3. **Enable GitHub Actions** in your repository settings

4. **Test the workflows** by pushing commits or creating pull requests

## Using the System

### For Development:
```bash
# Run tests locally
make test

# Build locally
make build

# Run all checks
make check
```

### For Releases:
```bash
# Create a release
make release patch  # or minor, major, or specific version

# This will:
# 1. Update Cargo.toml version
# 2. Update CHANGELOG.md
# 3. Create git tag
# 4. When pushed, trigger release workflow
```

The workflows will automatically handle:
- Multi-platform builds
- Binary artifact creation
- GitHub release creation
- Crates.io publishing (if configured)