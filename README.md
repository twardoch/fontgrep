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

## Dependencies

- ahash v0.8.11: Fast hash function, providing efficient hashing for data structures.
- aho-corasick v1.1.3: Implements Aho-Corasick algorithm for efficient string searching.
- allocator-API2 v0.2.21: Mirrors Rust's allocator API for stable Rust, enabling allocator features.
- anstream v0.6.18: Streams text with ANSI styles, useful for colored terminal output.
- anstyle v1.0.10: This crate provides a way to work with ANSI styles, including parsing and generating styled text.
- anstyle-parse v0.2.6: Parses ANSI style escape codes for terminal text styling.
- anstyle-query v1.1.2: Queries ANSI style attributes for manipulating terminal text styles.
- autocfg v1.4.0: Automatic configuration based on the platform, handling OS differences.
- bitflags v2.9.0: Handles bit flags, simplifying work with sets of flags represented by integers.
- bytemuck v1.22.0: Facilitates type casting without copying, useful for raw bytes manipulation.
- bytemuck_derive v1.8.1: Derive macro for bytemuck, handling type casting without copying.
- cc v1.2.16: Interacts with C compilers, aiding in compiling C code from Rust.
- cfg-if v1.0.0: Enables conditional compilation based on cargo features.
- clap v4.5.31: Command-line argument parsing library, offering simple and efficient parsing.
- clap_builder v4.5.31: Part of Clap, provides a fluent API for building command-line interfaces.
- clap_derive v4.5.28: Derive macro for clap, defining command-line interfaces with structs and enums.
- clap_lex v0.7.4: Part of Clap, handles lexical analysis for command-line argument parsing.
- colorchoice v1.0.3: Chooses colors for terminal output based on capabilities, ensuring compatibility.
- crossbeam-deque v0.8.6: Concurrent deque data structure, useful for producer-consumer scenarios.
- crossbeam-epoch v0.9.18: Supports epoch-based memory reclamation for concurrent data structures.
- crossbeam-utils v0.8.21: Utilities for concurrent programming, including thread management.
- dirs v5.0.1: Provides standard directories, like config or data, in a cross-platform way.
- dirs-sys v0.4.1: System-specific directories, finding standard paths like home or config.
- either v1.14.0: Works with values that can be one of two types, similar to Either in other languages.
- env_filter v0.1.3: Filters log messages based on environment variables, configuring logging verbosity.
- env_logger v0.11.7: Logger reading configuration from environment variables, simplifying logging setup.
- fallible-iterator v0.2.0: Another crate for iterators that can fail, similar to fallible-streaming-iterator.
- fallible-streaming-iterator v0.1.9: Provides iterators that can fail during iteration, handling errors in streams.
- font-types v0.8.3: Defines types for font handling, part of the Skrifa project.
- getrandom v0.3.1: Generates random numbers using the operating system's random generator.
- hashbrown v0.14.5: Fast hash map implementation, providing efficient key-value storage.
- hashlink v0.8.4: Linked hash map, maintaining insertion order with fast lookups.
- heck v0.5.0: Provides case conversion for strings, like snake_case to camelCase.
- is_terminal_polyfill v1.70.1: Polyfill for checking if stdout is a terminal, ensuring cross-platform behavior.
- itoa v1.0.15: Converts integers to strings efficiently, avoiding standard library formatting.
- jiff v0.2.4: Datetime library for Rust, emphasizing ease of use and safety in time handling.
- libc v0.2.170: Provides bindings to the C standard library for system-level programming.
- libsqlite3-sys v0.26.0: System bindings for SQLite, providing low-level access to the SQLite library.
- lock_api v0.4.12: Defines a trait for mutexes, used by synchronization crates like parking_lot.
- log v0.4.26: Logging facade, providing a common API for logging implementations.
- memchr v2.7.4: Implements memory character search, similar to C's memchr function.
- memmap2 v0.5.10: Memory maps files, allowing direct access to file data as if in memory.
- num_cpus v1.16.0: Determines the number of CPUs available, helpful for parallel processing.
- once_cell v1.20.3: Supports lazy initialization, ensuring single execution of initialization.
- option-ext v0.2.0: Extends the Option type with additional methods, enhancing usability.
- parking_lot v0.12.3: Provides efficient synchronization primitives like mutexes and condition variables.
- parking_lot_core v0.9.10: Core synchronization primitives for parking_lot, offering efficient mutexes.|
- pkg-config v0.3.32: Interacts with pkg-config to find and link external libraries.
- ppv-lite86 v0.2.21: SIMD-based hash function, providing fast hashing for performance-critical apps.
- proc-macro2 v1.0.94: Library for procedural macros, enabling code generation at compile time.
- quote v1.0.39: Token stream manipulation, used in macros for code generation.
- r2d2 v0.8.10: Generic connection pool, managing resources like database connections.
- r2d2_sqlite v0.22.0: Connection pool for SQLite using r2d2, managing database connections efficiently.
- rand v0.9.0: Random number generation library, offering a high-level API for random data.
- rand_chacha v0.9.0: Implements ChaCha random number generator, known for speed and security.
- rand_core v0.9.3: Core library for random number generation, providing traits and basics.
- rayon v1.10.0: Parallel iteration library, simplifying parallel processing of loops.
- rayon-core v1.12.1: Core library for Rayon, providing foundation for parallel iteration.
- read-fonts v0.27.2: Reads font files, providing low-level parsing of font data.
- regex v1.11.1: Regular expression library, providing powerful pattern matching capabilities.
- regex-automata v0.4.9: Provides regex matching capabilities, used by regex crate for efficiency.
- regex-syntax v0.8.5: Parses regex syntax, used by regex crate for compiling regular expressions.
- rusqlite v0.29.0: SQLite database library for Rust, enabling easy interaction with SQLite databases.
- ryu v1.0.20: Fast floating-point to string conversion, efficient for number formatting.
- same-file v1.0.6: Checks if two paths refer to the same file, useful for file system operations.
- scheduled-thread-pool v0.2.7: Manages a thread pool for scheduling tasks, useful for concurrent execution.
- scopeguard v1.2.0: Manages scope-based resource management, ensuring operations on scope exit.
- serde v1.0.219: Serialization and deserialization framework, handling data conversion.
- serde_derive v1.0.219: Derive macro for serde, simplifying serialization and deserialization implementation.
- serde_json v1.0.140: JSON serialization and deserialization, part of the serde ecosystem.
- shlex v1.3.0: Provides shell-like lexical analysis for parsing command-line text.
- skrifa v0.28.1: Robust, high-performance crate for OpenType fonts, handling metadata and glyphs.
- smallvec v1.14.0: This crate provides a vector-like data structure that stores its elements in a small buffer on the stack before switching to the heap when the buffer is full. It's useful for reducing memory allocations.
- strsim v0.11.1: Calculates string similarity, useful for fuzzy matching or spell checking.
- syn v2.0.99: Parses Rust code, used in derive macros and procedural macros.
- thiserror v1.0.69: Defines custom error types with derive macros, simplifying error handling.
- thiserror-impl v1.0.69: Implementation for thiserror, providing macros for custom error types.
- unicode-ident v1.0.18: Handles Unicode identifiers, ensuring proper non-ASCII character support.
- utf8parse v0.2.2: Parses UTF-8 strings, validating and manipulating encoded text.
- uuid v1.15.1: Generates and parses universally unique identifiers (UUIDs) for distributed systems.
- vcpkg v0.2.15: Integrates vcpkg dependencies into Rust projects for C/C++ library use.
- version_check v0.9.5: Checks crate versions for compatibility and correct usage.
- walkdir v2.5.0: Traverses directories, providing an iterator over files and subdirectories.
- zerocopy v0.8.23: Facilitates zero-copy operations for safe byte sequence conversions.

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Uses [skrifa](https://github.com/googlefonts/skrifa) for font parsing
- Uses [rusqlite](https://github.com/rusqlite/rusqlite) for SQLite database access
- Uses [clap](https://github.com/clap-rs/clap) for command-line argument parsing 