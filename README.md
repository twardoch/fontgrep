# fontgrep

A command-line tool to quickly find and filter fonts based on specific features, Unicode characters, variation axes, and more.

## Key Features

- **Variation Axes**: Find fonts with specific adjustable properties (e.g., weight, width).
- **OpenType Features**: Locate fonts supporting typographic features like small caps or ligatures.
- **Unicode Support**: Identify fonts containing specific characters or text.
- **Variable Fonts**: Search specifically for fonts that allow dynamic adjustments.
- **OpenType Tables**: Find fonts containing specific technical tables (e.g., color fonts).
- **Script Support**: Discover fonts supporting particular writing systems (e.g., Latin, Cyrillic).
- **Name Matching**: Use regular expressions to match font names.
- **Persistent Cache**: Speed up repeated searches with a persistent cache.
- **Silent Operation**: Runs quietly without unnecessary error messages.
- **Supported Formats**: Works with `.otf` and `.ttf` font files.

## Performance Optimizations

`fontgrep` efficiently handles large font collections by:

- Using a persistent cache to avoid repeated parsing of font files.
- Quickly filtering files by type before detailed analysis.
- Using parallel processing (multi-threading) for speed.
- Accessing font files efficiently via memory mapping.
- Applying filters from least to most computationally expensive.
- Displaying results immediately as they're found.

## Usage

```bash
fontgrep [OPTIONS] [DIRECTORY]
```

### Options

- `-a, --axis <AXIS>`: Specify variation axes (e.g., weight `wght`).
- `-u, --unicode <UNICODE>`: Unicode characters or ranges (e.g., `U+20AC`).
- `-t, --text <TEXT>`: Specific text to check font support.
- `-f, --feature <FEATURE>`: OpenType features (e.g., small caps `smcp`).
- `-v, --variable`: Only find variable fonts.
- `-T, --table <TABLE>`: Specific OpenType tables (e.g., color fonts `COLR`).
- `-s, --script <SCRIPT>`: Writing systems (e.g., Latin `latn`).
- `-n, --name <NAME>`: Font names matching regular expressions.
- `-c, --cache <CACHE>`: Use a persistent cache file or directory.

## Examples

### Basic Usage

Find all fonts in a directory:
```bash
fontgrep /path/to/fonts
```

### Using the Cache

Build and query a cache for a directory:
```bash
fontgrep -c /path/to/fonts
```

Query the cache without scanning the filesystem:
```bash
fontgrep -c ~/.fontgrep_cache.db -v
```

Update the cache and find variable fonts:
```bash
fontgrep -c /path/to/fonts -v
```

### Variable Fonts

Find variable fonts with weight and width axes:
```bash
fontgrep -v -a wght -a wdth /path/to/fonts
```

### Unicode and Text

Find fonts supporting emoji:
```bash
fontgrep -u U+1F600-U+1F64F /path/to/fonts
```

Find fonts supporting specific text:
```bash
fontgrep -t "Hello, こんにちは, Привет!" /path/to/fonts
```

### OpenType Features

Find fonts with small caps:
```bash
fontgrep -f smcp /path/to/fonts
```

### OpenType Tables

Find color fonts:
```bash
fontgrep -T COLR /path/to/fonts
```

### Script Support

Find fonts supporting Latin and Cyrillic:
```bash
fontgrep -s latn -s cyrl /path/to/fonts
```

### Font Name Matching

Find fonts with "Sans" in the name:
```bash
fontgrep -n "Sans" /path/to/fonts
```

### Complex Queries

Find variable fonts supporting Cyrillic with weight and width axes:
```bash
fontgrep -v -a wght -a wdth -s cyrl /path/to/fonts
```

### Advanced Usage

Find fonts suitable for multilingual European websites:
```bash
fontgrep -s latn -s grek -s cyrl -u U+20AC,U+00A3,U+00A5 -f liga -f onum /path/to/fonts
```

### Integration with Other Tools

Count variable fonts:
```bash
fontgrep -v /path/to/fonts | wc -l
```

Copy variable fonts to another directory:
```bash
fontgrep -v /path/to/fonts | xargs -I{} cp "{}" /path/to/variable_fonts/
```

## Cache Details

The cache stores metadata about font files to speed up repeated searches:

- **File Information**: Paths, modification times, and file sizes.
- **Font Features**: Axes, OpenType features, scripts, tables, and Unicode support.
- **Storage Location**: By default, the cache is stored in the user's data directory.
- **Cache Modes**:
  - **Query-only mode**: When `-c` points to a cache file, queries run against the cached data only.
  - **Scan-and-update mode**: When `-c` points to a directory, the directory is scanned, the cache is updated, and results are printed.

The cache is automatically maintained:
- New files are added to the cache
- Modified files are updated in the cache
- Deleted files are removed from the cache

## Building

Compile the tool with:
```bash
cargo build --release
```

## License

MIT 