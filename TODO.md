# Fontgrep Codebase Improvement Plan

This document outlines a detailed implementation plan for improving the fontgrep codebase, focusing on maintainability, performance, and idiomatic Rust practices. The plan is based on a thorough analysis of the current codebase structure and the transition from the original single-file implementation (old_main.rs) to the current modular structure.

## 1. Current Status Assessment

The fontgrep codebase has evolved from a single-file implementation (old_main.rs) to a modular structure with separate files for different concerns:

```
src/
├── lib.rs           # Library exports, error handling, and re-exports
├── font.rs          # Font information extraction and matching
├── fontinfo.rs      # Legacy font information extraction (to be consolidated)
├── cache.rs         # Database operations and caching
├── query.rs         # Query execution and font matching
├── utils.rs         # Utility functions and helpers
├── cli.rs           # Command-line interface and configuration
└── main.rs          # Entry point
```

While this modularization is a significant improvement, several areas still need attention:

1. **Duplication**: Functionality is split between `font.rs` and `fontinfo.rs`
2. **Error Handling**: Some code still uses `unwrap()`/`expect()` instead of proper error propagation
3. **Database Operations**: The cache implementation could benefit from connection pooling and prepared statements
4. **Memory Efficiency**: Charset generation and font loading could be optimized
5. **Concurrency**: The current approach to parallelism could be improved
6. **Testing**: The codebase lacks comprehensive tests

## 2. Specific Implementation Tasks

### 2.1. Consolidate Font Information Handling

- [ ] **Merge `fontinfo.rs` into `font.rs`**

  - Move all functionality from `fontinfo.rs` to `font.rs`
  - Ensure consistent error handling
  - Update all imports to reference the new location
  - Remove `fontinfo.rs` after migration is complete

- [ ] **Optimize charset generation**
  - Replace the linear scan from U+0001 to U+10FFFF with direct charmap iteration:

```rust
  pub fn create_charset(font: &FontRef) -> BTreeSet<u32> {
      font.charmap().iter()
          .filter(|(cp, _)| !is_invalid_unicode(*cp))
          .map(|(cp, _)| cp)
          .collect()
  }
```

- [ ] **Implement the `FontMatcher` trait for different criteria**
  - Create specialized matchers for each filter type (axes, features, scripts, etc.)
  - Ensure consistent matching behavior between direct font matching and cache queries

### 2.2. Improve Database Operations

- [ ] **Implement connection pooling**
  - Add r2d2 and r2d2_sqlite dependencies
  - Refactor `FontCache` to use a connection pool:

```rust
  pub struct FontCache {
      pool: Pool<SqliteConnectionManager>,
      path: PathBuf,
  }
```

- [ ] **Use prepared statements for repetitive operations**

  - Identify SQL statements that are executed frequently
  - Convert them to prepared statements to improve performance
  - Use parameter binding instead of string concatenation

- [ ] **Implement transaction guards**
  - Create a `TransactionGuard` struct for RAII-style transaction management:

```rust
  struct TransactionGuard<'a> {
      tx: Option<rusqlite::Transaction<'a>>,
  }

  impl<'a> Drop for TransactionGuard<'a> {
      fn drop(&mut self) {
          if let Some(tx) = self.tx.take() {
              tx.rollback().ok();
          }
      }
  }
```

- [ ] **Normalize the database schema**
  - Consider consolidating similar tables (axes, features, scripts) into a single table with a type column
  - Add appropriate indexes for common query patterns
  - Ensure proper foreign key constraints

### 2.3. Enhance Error Handling

- [ ] **Audit and replace all `unwrap()`/`expect()` calls**

  - Identify all instances in the codebase
  - Replace with proper error propagation using `?` operator
  - Ensure meaningful error messages

- [ ] **Expand the `FontgrepError` enum**

  - Add more specific error variants as needed
  - Implement appropriate `From` traits for common error types
  - Add context information to errors where helpful

- [ ] **Add error recovery strategies**
  - Implement graceful fallbacks for non-critical errors
  - Add retry logic for transient failures (e.g., database locks)
  - Provide helpful error messages to users

### 2.4. Optimize Performance

- [ ] **Improve parallelism in directory scanning**

  - Use Rayon's parallel iterators more effectively
  - Implement work stealing for better load balancing
  - Add configurable parallelism levels

- [ ] **Optimize memory usage**

  - Use memory mapping more efficiently
  - Consider lazy loading for large font data
  - Implement streaming for large result sets

- [ ] **Improve query performance**
  - Implement the query builder pattern for SQL generation
  - Add query plan analysis for complex queries
  - Consider caching query results for repeated queries

### 2.5. Enhance CLI Interface

- [ ] **Separate CLI handling from business logic**

  - Create a `Config` struct to encapsulate application settings
  - Implement conversion from CLI args to Config
  - Move command execution logic to dedicated functions

- [ ] **Improve subcommand structure**

  - Ensure consistent behavior across subcommands
  - Add detailed help text for each command
  - Implement validation for command arguments

- [ ] **Add output formatting options**
  - Support multiple output formats (text, JSON, CSV)
  - Implement pretty printing for human-readable output
  - Add machine-readable output for scripting

### 2.6. Add Comprehensive Testing

- [ ] **Add unit tests for core functionality**

  - Test font information extraction
  - Test query matching logic
  - Test cache operations

- [ ] **Add integration tests**

  - Test end-to-end workflows
  - Test CLI interface
  - Test with real font files

- [ ] **Add property-based tests**
  - Test with randomly generated inputs
  - Test edge cases
  - Test with large datasets

### 2.7. Improve Documentation

- [ ] **Add doc comments to all public items**

  - Document function parameters and return values
  - Explain complex algorithms
  - Add examples for common use cases

- [ ] **Update README with usage examples**

  - Add examples for common tasks
  - Document CLI options
  - Add installation instructions

- [ ] **Add CONTRIBUTING.md**
  - Document development workflow
  - Explain code organization
  - Set coding standards

## 3. Implementation Roadmap

### 3.1. Phase 1: Core Improvements (1-2 weeks)

1. Consolidate font information handling
2. Implement connection pooling
3. Enhance error handling
4. Add basic unit tests

### 3.2. Phase 2: Performance Optimization (2-3 weeks)

1. Optimize charset generation
2. Improve parallelism
3. Implement prepared statements
4. Add transaction guards

### 3.3. Phase 3: CLI and Documentation (1-2 weeks)

1. Enhance CLI interface
2. Improve documentation
3. Add integration tests
4. Update README

### 3.4. Phase 4: Refinement and Testing (1-2 weeks)

1. Add property-based tests
2. Optimize memory usage
3. Refine error messages
4. Final code cleanup

## 4. Migration Strategy

To ensure a smooth transition from the current codebase to the improved version:

1. **Incremental Changes**: Implement changes in small, testable increments
2. **Backward Compatibility**: Maintain API compatibility where possible
3. **Feature Flags**: Use feature flags for major changes to allow gradual adoption
4. **Comprehensive Testing**: Add tests before making significant changes

## 5. Success Metrics

- **Code Quality**:

  - Reduce code duplication by 40%
  - Eliminate all `unwrap()`/`expect()` calls
  - Pass `clippy` with no warnings

- **Performance**:

  - 3x faster queries for large font collections
  - 50% reduction in memory usage during scanning
  - 2x faster charset generation

- **Maintainability**:
  - 80%+ test coverage
  - Complete documentation for all public APIs
  - Clear separation of concerns in module structure

## 6. Specific Improvements from old_main.rs

Comparing the original old_main.rs with the current modular structure, these specific improvements should be made:

1. **Unicode Parsing**: Consolidate the unicode parsing logic from old_main.rs into a dedicated function in utils.rs

```rust
   // Move this function from old_main.rs to utils.rs
   pub fn parse_unicode_ranges(arg: &str) -> Result<Vec<u32>> {
       // Improved implementation with better error handling
   }
```

2. **Filter Functions**: Convert the filter functions from old_main.rs into proper FontMatcher implementations

```rust
   // Convert this from old_main.rs:
   fn feature_filter(font: &FontRef, feature: &str) -> bool { ... }

   // To this in font.rs:
   pub struct FeatureMatcher {
       features: Vec<String>,
   }

   impl FontMatcher for FeatureMatcher {
       fn matches(&self, info: &FontInfo) -> bool { ... }
   }
```

3. **Directory Walking**: Improve the directory walking logic from old_main.rs

```rust
   // Replace the WalkDir usage with a more efficient implementation
   pub fn scan_directories(dirs: &[PathBuf], jobs: usize) -> Result<Vec<PathBuf>> {
       // Parallel implementation using Rayon
   }
```

4. **Tag Parsing**: Move the tag parsing logic to utils.rs

```rust
   // Move this function from old_main.rs to utils.rs
   pub fn parse_font_tags(arg: &str) -> Result<Tag> {
       // Improved implementation with better error handling
   }
```

By implementing these specific improvements, the codebase will maintain the best aspects of the original implementation while benefiting from the modular structure and enhanced functionality.
