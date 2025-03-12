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
- Output in text or JSON format
- Detailed font information display

## Installation

```bash
cargo install fontgrep
```

## Usage

### Basic Font Searching

```bash
# Find variable fonts (without cache)
fontgrep find --variable /path/to/fonts

# Find fonts with specific features (without cache)
fontgrep find -f smcp,onum /path/to/fonts

# Find fonts supporting specific scripts (without cache)
fontgrep find -s latn,cyrl /path/to/fonts

# Find fonts by name pattern (without cache)
fontgrep find -n "Roboto.*Mono" /path/to/fonts

# Find fonts supporting specific Unicode ranges (without cache)
fontgrep find -u "U+0041-U+005A,U+0061-U+007A" /path/to/fonts

# Find fonts with specific tables (without cache)
fontgrep find -T GPOS,GSUB /path/to/fonts
```

### Fast Searching with Cache

```bash
# Fast search for variable fonts (with cache)
fontgrep fast --variable /path/to/fonts

# Fast search for fonts with specific features (with cache)
fontgrep fast -f smcp,onum /path/to/fonts
```

### Cache Management

```bash
# Save fonts to the cache
fontgrep save /path/to/fonts

# Force update the cache even if fonts haven't changed
fontgrep save --force /path/to/fonts

# List all saved fonts in the cache
fontgrep saved

# Remove missing fonts from the cache
fontgrep forget
```

### Font Information

```bash
# Show information about a font
fontgrep font /path/to/font.ttf

# Show detailed information about a font
fontgrep font -d /path/to/font.ttf
```

### Output Formats

```bash
# Output in JSON format
fontgrep find -j /path/to/fonts

# Output in JSON format (alternative)
fontgrep --json find /path/to/fonts
```

## Configuration

- **Cache Commands**: Use `fast` for cached searches and `find` for direct searches without cache.
- **Cache Location**: Use `--cache-path` to specify a custom cache location (defaults to user's data directory).
- **Parallel Jobs**: Use `-j/--jobs` to control the number of parallel jobs (defaults to CPU core count).
- **Verbose Output**: Use `-v/--verbose` for detailed logging.

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