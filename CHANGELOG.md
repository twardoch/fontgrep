# Changelog

## [0.2.0] - 2023-10-15

### Added
- New subcommand-based CLI interface with improved help messages
- Multiple output formats (text, JSON, CSV)
- Connection pooling for better database performance
- Unified property table for simpler schema
- Configurable number of parallel jobs
- Detailed font information display
- Comprehensive error handling with custom error types
- Unit tests for core functionality

### Changed
- Streamlined codebase with fewer files (7 source files total)
- Improved font information extraction using skrifa 0.15.0
- Enhanced cache implementation with better transaction handling
- More efficient charset handling
- Simplified font matching logic
- Better error messages and logging

### Fixed
- Memory leaks in font processing
- Race conditions in parallel processing
- Improved error handling for missing fonts
- Better handling of invalid Unicode codepoints

## [0.1.2] - 2023-09-01

### Added
- Initial public release
- Basic font searching functionality
- Support for OpenType features, scripts, and tables
- Variable font detection
- Unicode codepoint matching
- Simple caching mechanism 