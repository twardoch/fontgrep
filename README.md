# fontgrep

A command-line tool for searching and analyzing font files based on various criteria.

## Features

- Search for fonts based on:
  - OpenType variation axes (e.g., weight, width)
  - OpenType features (e.g., small caps, old-style numerals)
  - OpenType scripts (e.g., Latin, Cyrillic)
  - Font tables (e.g., GPOS, GSUB)
  - Unicode character support
  - Font name patterns
- Fast searching with SQLite-based caching
- Parallel processing for improved performance
- Multiple output formats (text, JSON, CSV)
- Detailed font information display

## Installation

```bash
cargo install fontgrep
```

## Usage

### Basic Search

```bash
# Search for variable fonts
fontgrep search --variable /path/to/fonts

# Search for fonts with specific features
fontgrep search -f smcp,onum /path/to/fonts

# Search for fonts supporting specific scripts
fontgrep search -s latn,cyrl /path/to/fonts

# Search for fonts by name pattern
fontgrep search -n "Roboto.*Mono" /path/to/fonts

# Search for fonts supporting specific Unicode ranges
fontgrep search -u "U+0041-U+005A,U+0061-U+007A" /path/to/fonts
```

### Cache Management

```bash
# Update the font cache
fontgrep update /path/to/fonts

# Clean the cache (remove missing fonts)
fontgrep clean

# List all cached fonts
fontgrep list
```

### Font Information

```bash
# Show detailed information about a font
fontgrep info -d /path/to/font.ttf
```

### Output Formats

```bash
# Output in JSON format
fontgrep search --format json /path/to/fonts

# Output in CSV format
fontgrep search --format csv /path/to/fonts
```

## Configuration

- Cache location: By default, the cache is stored in the user's data directory. Use `--cache` to specify a custom location.
- Parallel jobs: Use `-j/--jobs` to control the number of parallel jobs (defaults to CPU core count).
- Verbose output: Use `-v/--verbose` for detailed logging.

## Performance

- Uses memory mapping for efficient font file access
- Maintains a SQLite cache with optimized indices
- Employs parallel processing for searching and font analysis
- Uses WAL mode for improved database concurrency

## Error Handling

- Graceful error recovery with detailed error messages
- Proper cleanup of resources
- Comprehensive logging for debugging

## Development

### Building from Source

```bash
git clone https://github.com/twardoch/fontgrep.git
cd fontgrep
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Uses [skrifa](https://github.com/googlefonts/skrifa) for font parsing
- Uses [rusqlite](https://github.com/rusqlite/rusqlite) for SQLite database access
- Uses [clap](https://github.com/clap-rs/clap) for command-line argument parsing 