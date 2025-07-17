# fontgrep

[![CI](https://github.com/twardoch/fontgrep/workflows/CI/badge.svg)](https://github.com/twardoch/fontgrep/actions)
[![Crates.io](https://img.shields.io/crates/v/fontgrep.svg)](https://crates.io/crates/fontgrep)
[![Downloads](https://img.shields.io/crates/d/fontgrep.svg)](https://crates.io/crates/fontgrep)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Codecov](https://codecov.io/gh/twardoch/fontgrep/branch/main/graph/badge.svg)](https://codecov.io/gh/twardoch/fontgrep)

A command-line tool for searching and analyzing font files based on various criteria.

## Features

- Search for fonts based on:
  - OpenType variation axes (e.g., weight, width)
  - OpenType features (e.g., small caps, old-style numerals)
  - OpenType scripts (e.g., Latin, Cyrillic)
  - Font tables (e.g., GPOS, GSUB)
  - Unicode character support
  - Font name patterns
- Progressive output for immediate feedback
- Parallel processing for improved performance
- Output in text or JSON format

## Installation

### Quick Install

```bash
# From crates.io
cargo install fontgrep

# From source
git clone https://github.com/twardoch/fontgrep.git
cd fontgrep
cargo install --path .
```

### Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/twardoch/fontgrep/releases):

- **Linux**: `fontgrep-linux-x86_64.tar.gz`
- **macOS**: `fontgrep-macos-x86_64.tar.gz` (Intel) or `fontgrep-macos-aarch64.tar.gz` (Apple Silicon)
- **Windows**: `fontgrep-windows-x86_64.exe.zip`

### Docker

```bash
docker run --rm -v $(pwd):/fonts ghcr.io/twardoch/fontgrep:latest /fonts
```

For detailed installation instructions, see [INSTALL.md](INSTALL.md).

## Usage

### Basic Font Searching

```bash
# Find variable fonts
fontgrep --variable /path/to/fonts

# Find fonts with specific features
fontgrep -f smcp,onum /path/to/fonts

# Find fonts supporting specific scripts
fontgrep -s latn,cyrl /path/to/fonts

# Find fonts by name pattern
fontgrep -n "Roboto.*Mono" /path/to/fonts

# Find fonts supporting specific Unicode ranges
fontgrep -u "U+0041-U+005A,U+0061-U+007A" /path/to/fonts

# Find fonts with specific tables
fontgrep -T GPOS,GSUB /path/to/fonts

# Find fonts supporting specific text
fontgrep -t "Hello World" /path/to/fonts
```

### Combining Search Criteria

```bash
# Find variable fonts with small caps feature
fontgrep --variable -f smcp /path/to/fonts

# Find fonts supporting both Latin and Cyrillic scripts
fontgrep -s latn,cyrl /path/to/fonts

# Find fonts with specific name pattern and features
fontgrep -n "Roboto" -f liga,kern /path/to/fonts
```

### Output Formats

```bash
# Output in JSON format
fontgrep -j /path/to/fonts

# Combine JSON output with search criteria
fontgrep -j -f smcp,onum /path/to/fonts
```

## Command-Line Options

- `-a, --axes <AXES>`: Comma-separated list of OpenType variation axes to search for (e.g., wght,wdth)
- `-f, --features <FEATURES>`: Comma-separated list of OpenType features to search for (e.g., smcp,onum)
- `-s, --scripts <SCRIPTS>`: Comma-separated list of OpenType script tags to search for (e.g., latn,cyrl)
- `-T, --tables <TABLES>`: Comma-separated list of OpenType table tags to search for (e.g., GPOS,GSUB)
- `-v, --variable`: Only show variable fonts that support OpenType Font Variations
- `-n, --name <NAME>`: Regular expressions to match against font names
- `-u, --codepoints <CODEPOINTS>`: Unicode codepoints or ranges to search for (e.g., U+0041-U+005A,U+0061)
- `-t, --text <TEXT>`: Text string to check for support
- `-J, --jobs <JOBS>`: Number of parallel jobs to use (defaults to CPU core count)
- `--verbose`: Enable verbose output
- `-j, --json`: Output results in JSON format
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Performance

- Uses memory mapping for efficient font file access
- Employs parallel processing for searching and font analysis
- Provides progressive output for immediate feedback

## Development

### Building from Source

```bash
git clone https://github.com/twardoch/fontgrep.git
cd fontgrep
make build
```

### Running Tests

```bash
# Run all tests
make test

# Run specific test types
make test-unit
make test-integration
make test-doc
```

### Development Workflow

```bash
# Set up development environment
make setup

# Run quick development checks
make check

# Run full CI pipeline locally
make ci

# Generate code coverage
make coverage

# Run security audit
make audit

# Check binary size
make size
```

### Release Process

```bash
# Prepare a new release
make release patch  # or minor, major, or specific version

# The release script will:
# 1. Run tests and build
# 2. Update version in Cargo.toml
# 3. Update CHANGELOG.md
# 4. Create git tag
# 5. Push to trigger CI/CD pipeline
```

### Docker Development

```bash
# Build Docker image
make docker-build

# Run with testdata
make docker-run
```

### Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`make test`)
5. Run quality checks (`make check`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Available Make Targets

Run `make help` to see all available targets.

### GitHub Actions Setup

Due to permission requirements, GitHub Actions workflows need to be set up manually. See [WORKFLOWS.md](WORKFLOWS.md) for detailed instructions on setting up CI/CD pipelines for automated testing, building, and releasing.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Uses [skrifa](https://github.com/googlefonts/skrifa) for font parsing
- Uses [clap](https://github.com/clap-rs/clap) for command-line argument parsing
- Uses [jwalk](https://github.com/jessegrosjean/jwalk) for parallel directory traversal 