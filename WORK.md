# TODO 1

## 1. Improvement Proposal

1. **Refactor Common Query Logic**

   - **Reason**: Many SQL statements in `cache.rs` and parts of `query.rs` share repeated clauses.
   - **Plan**:

     - Extract repeated code (like “list all fonts,” “fetch font_id,” etc.) into helper functions or trait methods.
     - Encapsulate condition-building logic for queries to reduce the string manipulation and potential for mistakes.

2. **Use Stronger Types / Enums for Filter Criteria**

   - **Reason**: Currently, filters (axes, features, scripts) are all handled as `String` or `Tag` . More explicit types can reduce bugs and make code more “Rustonic.”
   - **Plan**:

     - Define an enum or typed struct to represent each query criterion (axis, feature, etc.).
     - Implement `Display` and conversion traits to handle bridging to SQL or string queries.

3. **Improve Error Handling**

   - **Reason**: Some code uses `match` on results but still relies heavily on `unwrap()` . Using `?` more consistently and returning well-defined errors improves reliability.
   - **Plan**:

     - Replace direct `unwrap()` calls with `?` or custom error types (possibly via `anyhow` or `thiserror` for simpler error definitions).
     - Provide meaningful error messages (especially for file I/O or parse failures).

4. **Batch Operations & Transactions**

   - **Reason**: There is a good start on batch updates with `batch_update_fonts` , but the rest of the code can unify with batch strategies to reduce repeated database hits and partial transaction states.
   - **Plan**:

     - Use the single transaction approach for all updating operations when possible.
     - For smaller updates (like single-file updates), keep them in a well-scoped transaction.
     - Move transaction logic out of large methods to specialized functions or macros so that the logic is not repeated.

5. **Use Parallelism Carefully**

   - **Reason**: The code uses Rayon in scanning directories. Ensure no hidden pitfalls with concurrency (like lock contention on the cache or `Arc<Mutex<…>>` ).
   - **Plan**:

     - Consider finer-grained concurrency or keep concurrency to the scanning phase while ensuring the database writes remain thread-safe.
     - Possibly queue updates for the database in a channel that a single thread consumes to avoid repeated lock/unlock overheads.

6. **Consolidate Codepoint / Unicode Logic**

   - **Reason**: The code has repeated logic to decide whether to parse short text vs. large ranges. Some of this is scattered between `main.rs` , `query.rs` , and `fontinfo.rs` .
   - **Plan**:

     - Introduce a small module or functions dedicated to parsing and analyzing Unicode sets, returning a `HashSet<u32>` or similar.
     - Make the code simpler by reducing branching for short vs. long text in multiple places.

7. **Enhance Testing & Stability**

   - **Reason**: Code with lots of queries benefits greatly from more thorough unit/integration tests.
   - **Plan**:

     - Add tests covering “found font,” “no match,” “partial match,” etc.
     - Test caching logic for correctness when updating or removing fonts.

8. **Optional**: Simplify “create_charset_string” Implementation

   - **Reason**: Currently it checks every codepoint up to `0x10FFFF` , which is expensive.
   - **Plan**:

     - Consider building a `BTreeSet` of supported codepoints from the font’s built-in coverage methods, or skipping codepoints above a certain threshold if it harms performance.
     - Verify correctness if changes are introduced (or provide a config to toggle full scanning vs. partial scanning).

## 2. Summary

By adopting these changes—removing repeated logic, introducing stronger data types, refining error handling, carefully using concurrency, and adding better tests—the codebase will become more maintainable, efficient, and idiomatic to Rust (“Rustonic”). The above items are designed to be incremental steps so they can be tackled one at a time rather than a single large refactor.

====

# TODO 2

Below is an improvement proposal for the `fontgrep` codebase, designed to enhance maintainability, reduce convolution, increase modularity, align with Rust idioms ("more Rustonic"), boost efficiency, eliminate duplication, improve safety, and enhance stability. This proposal is written as a `TODO.md` file, providing actionable tasks based on a detailed analysis of the existing code.

---

# TODO.md - Improvement Proposal for fontgrep

## 3. Overview

This document outlines improvements to the `fontgrep` codebase to make it more maintainable, efficient, modular, idiomatic to Rust, and robust. The tool, a command-line utility for searching font directories based on criteria like variation axes, Unicode support, and OpenType features, is already well-structured but can benefit from targeted enhancements.

## 4. Key Improvements

### 4.1. Error Handling

- **Current State**: The codebase relies heavily on `unwrap` and `expect`, risking panics on failures (e.g., file operations in `query.rs`, database locks in `cache.rs`).
- **Proposal**: Replace these with proper error propagation using `Result` and the `?` operator, returning meaningful errors to the caller.
- **Benefit**: Prevents crashes, improves user feedback, and aligns with Rust's safety guarantees.

### 4.2. Modularization

- **Current State**: Functions like `FontQuery::execute` in `query.rs` and `FontCache::update_font` in `cache.rs` are lengthy and handle multiple responsibilities (e.g., cache querying vs. directory searching).
- **Proposal**: Break these into smaller, focused functions, such as separating cache querying and directory searching in `execute`.
- **Benefit**: Enhances readability, simplifies debugging, and makes future extensions easier.

### 4.3. Centralize Font Parsing

- **Current State**: Font file opening, memory mapping, and parsing logic is duplicated in `query.rs` (e.g., `search_directory`, `extract_font_info`) and `cache.rs` (implied in `update_font` usage).
- **Proposal**: Create a `font_utils.rs` module with a reusable `load_font` function to standardize font loading.
- **Benefit**: Reduces code duplication, centralizes error handling, and ensures consistent font processing.

### 4.4. Optimize Database Operations

- **Current State**: Batch updates in `cache.rs` use a fixed batch size of 50, and SQL queries in `FontCache::query` dynamically build complex JOINs, which may scale poorly with many criteria.
- **Proposal**: Make batch size configurable or dynamic (e.g., based on system resources), and explore precompiled SQL statements or query optimizations.
- **Benefit**: Improves performance for large font collections and reduces database overhead.

### 4.5. Add Tests

- **Current State**: No tests are present, risking regressions as the code evolves.
- **Proposal**: Add unit tests for key functions (e.g., `font_matches`, `FontInfo::from_font`) and integration tests for CLI workflows using sample fonts.
- **Benefit**: Ensures correctness, facilitates refactoring, and improves long-term stability.

### 4.6. Improve Documentation

- **Current State**: Code lacks inline comments and docstrings, though `README.md` provides usage details.
- **Proposal**: Add Rust doc comments (`///`) to public functions and modules, detailing purpose, parameters, and return values.
- **Benefit**: Enhances maintainability for contributors and clarifies intent for complex logic.

### 4.7. Review CLI Logic

- **Current State**: CLI handling in `main.rs` uses complex logic for cache and directory options (e.g., `-c`, `-C`), which can confuse users and developers.
- **Proposal**: Simplify or clearly document the logic for determining search directories and cache behavior.
- **Benefit**: Reduces confusion, improves usability, and makes the code easier to maintain.

### 4.8. Profile and Optimize

- **Current State**: Performance relies on parallel processing (via `rayon`) and memory mapping, but bottlenecks (e.g., font parsing, database queries) are unprofiled.
- **Proposal**: Profile with large font collections to identify slow paths and optimize (e.g., batch font parsing, reduce lock contention).
- **Benefit**: Ensures scalability and responsiveness for real-world use cases.

## 5. Specific Tasks

### 5.1. Error Handling

- [ ] Replace `unwrap` and `expect` with `Result` and `?` in `cache.rs` (e.g., `FontCache::needs_update`, mutex locks).
- [ ] Update `query.rs` to propagate I/O and font parsing errors properly (e.g., `search_directory`, `extract_font_info`).
- [ ] Refactor `main.rs` to handle cache initialization errors gracefully instead of printing warnings.

### 5.2. Modularization

- [ ] Split `FontQuery::execute` in `query.rs` into:
  - `query_cache` for cache-based searches.
  - `search_directories` for direct directory searches.
- [ ] Refactor `FontCache::update_font` in `cache.rs` into smaller functions for updating specific metadata (e.g., `update_axes`, `update_features`).
- [ ] Move `is_font_file` from `query.rs` to a utility module for reuse across the codebase.

### 5.3. Centralize Font Parsing

- [ ] Create `src/font_utils.rs` with:

```rust
  use std::{fs::File, io, path::Path};
  use memmap2::Mmap;
  use skrifa::FontRef;

  pub fn load_font(path: &Path) -> io::Result<FontRef> {
      let file = File::open(path)?;
      let data = unsafe { Mmap::map(&file)? };
      FontRef::new(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
  }
```

- [ ] Replace duplicated font loading in `query.rs` (`search_directory`, `extract_font_info`) and `cache.rs` fallbacks with `load_font`.

### 5.4. Optimize Database Operations

- [ ] Make `BATCH_SIZE` in `FontQuery::update_cache_for_directory` configurable via a constant or CLI option.
- [ ] Investigate prepared statements for repetitive inserts in `FontCache::batch_update_fonts`.
- [ ] Profile `FontCache::query` with multiple criteria to assess JOIN performance; consider subqueries if needed.

### 5.5. Add Tests

- [ ] Write unit tests in `tests/`:
  - `fontinfo.rs`: Test `FontInfo::from_font` with a sample font file.
  - `query.rs`: Test `font_matches` with known font properties.
  - `cache.rs`: Test `needs_update` and `query` with mock data.
- [ ] Add integration tests in `tests/`:
  - Simulate CLI runs with temporary directories and sample fonts (e.g., `fontgrep -v`).
  - Verify cache update and query behavior.

### 5.6. Improve Documentation

- [ ] Add doc comments to `FontCache` and its methods in `cache.rs` (e.g., explain `query` parameters).
- [ ] Document `FontQuery` methods in `query.rs` (e.g., detail `execute` behavior with/without cache).
- [ ] Update `README.md` with any CLI logic changes for consistency.

### 5.7. Review CLI Logic

- [ ] Clarify `search_directories` logic in `main.rs` with comments or simplify flag handling (e.g., separate `--cache-path` and `--use-cache`).
- [ ] Validate `-c` and `-C` behavior aligns with `README.md` (e.g., cache update vs. query mode).

### 5.8. Profile and Optimize

- [ ] Use a profiling tool (e.g., `cargo flamegraph`) on a large font set to identify bottlenecks.
- [ ] Optimize font parsing in `fontinfo.rs` (e.g., lazy charset computation if unused).
- [ ] Minimize `Arc<Mutex>` lock durations in `query.rs` and `cache.rs` (e.g., batch lock acquisitions).

---

## 6. Conclusion

These improvements balance immediate gains (e.g., error handling, modularity) with long-term benefits (e.g., tests, optimization). Prioritizing error handling, modularization, and tests will yield the most impact for maintainability and stability, while profiling ensures efficiency as the tool scales. Each task is actionable and leverages Rust’s strengths, making `fontgrep` a more robust and idiomatic project.

# TODO 3

# Analyzing the Fontgrep Codebase for Improvement

I'll analyze the fontgrep codebase to identify areas for improvement, focusing on making it more maintainable, modular, efficient, and idiomatic Rust.

## 7. Step-by-Step Analysis

### 7.1. Understanding the Codebase Structure

The codebase consists of four main Rust files:

- `main.rs`: Entry point with CLI argument parsing and execution
- `query.rs`: Font query implementation
- `fontinfo.rs`: Font information extraction
- `cache.rs`: SQLite-based caching system

The application is a command-line tool for searching and filtering font files based on various criteria like OpenType tables, features, variable font axes, and Unicode support.

### 7.2. Identifying Key Issues

1. **Code Duplication**: There's significant duplication in query logic between different functions.

2. **Error Handling**: Error handling is inconsistent, with some functions returning `io::Result` and others returning `SqlResult`.

3. **Modularity**: The codebase has some large functions that could be broken down into smaller, more focused components.

4. **Memory Efficiency**: Some operations could be optimized to reduce memory usage, especially when processing large font collections.

5. **Rustic Idioms**: Several areas could benefit from more idiomatic Rust patterns.

6. **Concurrency Model**: The current approach to parallelism could be improved.

7. **Configuration Management**: CLI arguments are directly used throughout the code rather than being transformed into a more structured configuration.

### 7.3. Detailed Analysis of Specific Components

#### 7.3.1. Cache Implementation

- The `FontCache` struct in `cache.rs` has many large methods with similar SQL operations
- SQL queries are constructed using string concatenation, which is error-prone
- The cache uses a global lock for all operations, which could be a bottleneck

#### 7.3.2. Query Execution

- The `FontQuery` struct in `query.rs` has duplicated logic for searching directories and querying the cache
- Error handling is inconsistent between cache and filesystem operations
- The parallelism model uses raw thread synchronization primitives rather than more structured approaches

#### 7.3.3. Font Information Extraction

- The `FontInfo` struct in `fontinfo.rs` is well-structured but could benefit from more efficient string handling
- Character set extraction creates large strings that could be optimized

#### 7.3.4. CLI Interface

- The `main.rs` file mixes argument parsing with business logic
- There's no clear separation between the CLI interface and the core functionality

Now I'll write an improvement proposal based on this analysis.

### 7.4. TODO.md

````markdown
# Improvement Proposal for Fontgrep

This document outlines a comprehensive plan to improve the fontgrep codebase, making it more maintainable, modular, efficient, and idiomatic Rust.

## 8. Architectural Improvements

### 8.1. 1.1 Modular Architecture

- **Implement a clear domain model**: Separate the core domain logic from infrastructure concerns

  - Create a `domain` module with the core entities and business logic
  - Move infrastructure concerns (cache, file system) to an `infrastructure` module

- **Introduce a service layer**: Add a service layer between the CLI and the domain model
  - Create a `FontService` that orchestrates operations between components
  - Implement the Command pattern for different operations (search, cache update)

### 8.2. 1.2 Configuration Management

- **Create a dedicated configuration module**: Replace direct CLI argument usage with a structured configuration
  - Implement a `Config` struct that encapsulates all application settings
  - Add validation logic to ensure configuration consistency
  - Support loading configuration from environment variables and config files

## 9. Code Quality Improvements

### 9.1. 2.1 Error Handling

- **Implement a consistent error handling strategy**:
  - Create a custom `Error` enum that wraps all possible error types
  - Use `thiserror` for deriving error implementations
  - Replace all `io::Result` and `SqlResult` with the custom error type
  - Add context to errors using `anyhow` or similar

### 9.2. 2.2 Reduce Code Duplication

- **Extract common functionality into reusable components**:
  - Create a `FontMatcher` trait for different matching strategies
  - Implement the Strategy pattern for different query types
  - Extract SQL query building into a dedicated query builder

### 9.3. 2.3 Improve Testability

- **Increase test coverage**:

  - Add unit tests for core functionality
  - Implement integration tests for end-to-end scenarios
  - Use test fixtures for font files

- **Make components more testable**:
  - Use dependency injection for external dependencies
  - Implement traits for components to allow mocking

## 10. Performance Optimizations

### 10.1. 3.1 Memory Efficiency

- **Optimize memory usage**:
  - Replace string concatenation with more efficient string building
  - Use `SmallVec` for small collections to avoid heap allocations
  - Implement streaming for large result sets

### 10.2. 3.2 Concurrency Improvements

- **Modernize concurrency model**:
  - Replace raw `Arc<Mutex<>>` with more appropriate concurrency primitives
  - Use `tokio` or `async-std` for asynchronous operations
  - Implement a work-stealing thread pool for better load balancing

### 10.3. 3.3 Cache Optimizations

- **Improve cache performance**:
  - Use prepared statements consistently for all SQL operations
  - Implement connection pooling for database access
  - Add cache invalidation strategies for better consistency
  - Use read-write locks instead of mutexes where appropriate

## 11. Rustic Idioms

### 11.1. 4.1 Type Safety

- **Leverage Rust's type system**:
  - Replace string-based identifiers with newtype patterns
  - Use `PhantomData` for type-level constraints
  - Implement the `From`/`Into` traits for clean conversions

### 11.2. 4.2 Modern Rust Features

- **Use modern Rust features**:
  - Replace manual error propagation with the `?` operator
  - Use `impl Trait` for cleaner function signatures
  - Leverage `async`/`await` for I/O-bound operations

### 11.3. 4.3 API Design

- **Improve API ergonomics**:
  - Use the builder pattern for complex object construction
  - Implement iterator adaptors for processing collections
  - Use const generics for compile-time optimizations

## 12. Implementation Plan

### 12.1. 5.1 Phase 1: Refactoring

1. Create the domain model and service layer
2. Implement the custom error type
3. Extract common functionality into reusable components

### 12.2. 5.2 Phase 2: Performance Optimizations

1. Optimize memory usage
2. Modernize the concurrency model
3. Improve cache performance

### 12.3. 5.3 Phase 3: API Improvements

1. Implement the builder pattern for query construction
2. Add iterator adaptors for result processing
3. Improve CLI interface

## 13. Specific Code Changes

### 13.1. 6.1 Cache Module

```rust
// Replace this:
pub struct FontCache {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

// With this:
pub struct FontCache {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    path: PathBuf,
}
```
````

### 13.2. 6.2 Query Module

```rust
// Replace this:
pub struct FontQuery {
    // fields...
    cache: Option<Arc<Mutex<FontCache>>>,
}

// With this:
pub struct FontQuery {
    // fields...
    cache_service: Option<CacheService>,
}

// Add this:
pub trait FontMatcher {
    fn matches(&self, font: &FontRef) -> bool;
}

pub struct AxisMatcher(String);
pub struct FeatureMatcher(String);
// etc.

impl FontMatcher for AxisMatcher {
    fn matches(&self, font: &FontRef) -> bool {
        font.axes().iter().any(|a| a.tag().to_string() == self.0)
    }
}
```

### 13.3. 6.3 Main Module

```rust
// Replace direct CLI execution with:
fn main() -> Result<(), Error> {
    let args = Args::parse();
    let config = Config::from_args(args)?;

    let font_service = FontService::new(config.clone());

    match config.mode {
        Mode::Search => font_service.search()?,
        Mode::CacheUpdate => font_service.update_cache()?,
        Mode::CacheQuery => font_service.query_cache()?,
    }

    Ok(())
}
```

### 13.4. 6.4 FontInfo Module

```rust
// Replace string concatenation with:
fn extract_name_string(font: &FontRef) -> String {
    let mut name_strings = HashSet::new();

    if let Ok(name) = font.name() {
        for record in name.name_record() {
            if let Ok(string) = record.string(name.string_data()) {
                name_strings.insert(string.to_string());
            }
        }
    }

    // Use join instead of collecting to a Vec first
    name_strings.into_iter().fold(String::new(), |mut acc, s| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(&s);
        acc
    })
}
```

````

This improvement proposal addresses all the key issues identified in the codebase analysis and provides a clear path forward for making the fontgrep tool more maintainable, modular, efficient, and idiomatic Rust.

# TODO 4

# TODO.md: Code Improvement Proposal for FontGrep

## 14. Database Layer Improvements

### 14.1. 1.1 Schema Normalization
- **Problem:** Duplicated table structures for axes/features/scripts/tables
- **Solution:** Create unified `font_properties` table:
  ```sql
  CREATE TABLE font_properties (
    font_id INTEGER NOT NULL,
    prop_type TEXT NOT NULL CHECK(prop_type IN ('axis','feature','script','table')),
    tag TEXT NOT NULL,
    PRIMARY KEY (font_id, prop_type, tag),
    FOREIGN KEY (font_id) REFERENCES fonts(font_id) ON DELETE CASCADE
  )
````

### 14.2. 1.2 Connection Management

```rust
// Current
Arc>

// Proposed: Use connection pool
struct FontCache {
    pool: r2d2::Pool
}
```

### 14.3. 1.3 Query Builder Pattern

```rust
struct QueryBuilder {
    query: String,
    params: Vec>,
    join_counter: usize
}

impl QueryBuilder {
    fn with_property(mut self, prop_type: &str, values: &[String]) -> Self {
        values.iter().for_each(|v| {
            self.query += &format!(" JOIN font_properties p{}", self.join_counter);
            self.params.push(Box::new(prop_type));
            self.params.push(Box::new(v));
            self.join_counter += 1;
        })
        self
    }
}
```

## 15. Font Processing Optimization

### 15.1. 2.1 Charset Generation

- **Problem:** Linear scan from U+0001 to U+10FFFF is slow
- **Solution:** Use font's charmaps iterator:

```rust
fn create_charset_string(font: &FontRef) -> String {
    font.charmap().iter()
        .filter(|(cp, _)| !is_invalid_unicode(*cp))
        .map(|(cp, _)| cp)
        .collect()
}
```

### 15.2. 2.2 Parallel Processing Pipeline

```rust
rayon::scope(|s| {
    s.spawn(|_| process_axes(font));
    s.spawn(|_| process_features(font));
    s.spawn(|_| process_scripts(font));
    s.spawn(|_| process_tables(font));
})
```

## 16. Error Handling Overhaul

### 16.1. 3.1 Custom Error Type

```rust
#[derive(thiserror::Error, Debug)]
pub enum FontError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Font parsing error")]
    FontParse,

    #[error("Cache validation failed")]
    CacheInvalid,
}
```

### 16.2. 3.2 Result Type Alias

```rust
pub type FontResult = Result;
```

## 17. Cache Management Improvements

### 17.1. 4.1 Bulk Insert Optimization

```rust
fn batch_insert(
    tx: &Transaction,
    font_id: i64,
    prop_type: &str,
    tags: &[String]
) -> FontResult {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO font_properties VALUES (?1, ?2, ?3)"
    )?;

    tags.iter()
        .map(|tag| stmt.execute(params![font_id, prop_type, tag]))
        .collect::>()?;

    Ok(())
}
```

### 17.2. 4.2 Cache Validation

```rust
impl FontCache {
    fn validate_integrity(&self) -> FontResult {
        self.conn.query_row(
            "PRAGMA integrity_check",
            [],
            |row| {
                let status: String = row.get(0)?;
                if status == "ok" { Ok(()) }
                else { Err(FontError::CacheInvalid) }
            }
        )
    }
}
```

## 18. Code Organization

### 18.1. 5.1 Module Structure

```
src/
  core/
    cache/
      mod.rs       # Main cache logic
      schema.rs    # DB schema management
      queries.rs   # Query builders
    font/
      mod.rs       # Font processing
      features.rs  # Feature extraction
      charset.rs   # Charset handling
  cli/
    mod.rs        # CLI argument parsing
    commands/     # Subcommands
      query.rs
      cache.rs
```

### 18.2. 5.2 Trait-based Processing

```rust
trait FontProcessor {
    fn process_axes(&self, font: &FontRef) -> Vec;
    fn process_features(&self, font: &FontRef) -> Vec;
}

struct DefaultProcessor;
impl FontProcessor for DefaultProcessor {
    /* default implementations */
}

struct FastProcessor;
impl FontProcessor for FastProcessor {
    /* optimized implementations */
}
```

## 19. Performance Enhancements

### 19.1. 6.1 Memory Mapping Strategy

```rust
enum FontData {
    Mapped(Mmap),
    Buffered(Vec),
}

impl FontData {
    fn load(path: &Path) -> FontResult {
        match unsafe { Mmap::map(&File::open(path)?) } {
            Ok(mmap) => Ok(Self::Mapped(mmap)),
            Err(_) => Ok(Self::Buffered(fs::read(path)?))
        }
    }
}
```

### 19.2. 6.2 Query Plan Analysis

```rust
impl FontCache {
    fn analyze_query_performance(&self) -> FontResult {
        self.conn.execute_batch("
            ANALYZE;
            PRAGMA optimize;
        ")
    }
}
```

## 20. Safety Improvements

### 20.1. 7.1 Memory Mapping Wrapper

```rust
struct SafeMmap {
    file: File,
    mmap: Mmap,
}

impl SafeMmap {
    fn new(path: &Path) -> FontResult {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { file, mmap })
    }
}
```

### 20.2. 7.2 Transaction Guards

```rust
struct TransactionGuard {
    tx: Transaction,
    committed: bool,
}

impl TransactionGuard {
    fn commit(mut self) -> FontResult {
        self.tx.commit()?;
        self.committed = true;
        Ok(())
    }
}

impl Drop for TransactionGuard {
    fn drop(&mut self) {
        if !self.committed {
            self.tx.rollback().ok();
        }
    }
}
```

## 21. Testing Strategy

### 21.1. 8.1 Property-based Tests

```rust
proptest! {
    #[test]
    fn test_cache_roundtrip(path in ".*", mtime in any::(), size in any::()) {
        let cache = FontCache::memory()?;
        cache.update_font(&path, &dummy_font_info(), mtime, size)?;
        assert!(cache.get_font_id(&path, mtime, size)? > 0);
    }
}
```

### 21.2. 8.2 Benchmark Suite

```rust
#[bench]
fn bench_large_cache_query(b: &mut Bencher) {
    let cache = setup_100k_font_cache();
    b.iter(|| {
        cache.query(/* complex query */);
    });
}
```

## 22. **Implementation Roadmap**

| Priority | Component                 | Estimated Effort | Owner  |
| -------- | ------------------------- | ---------------- | ------ |
| P0       | Database Schema Migration | 3d               | Core   |
| P1       | Error Handling System     | 2d               | Safety |
| P2       | Query Builder Pattern     | 1.5d             | Query  |
| P3       | Parallel Processing       | 2d               | Perf   |
| P4       | Cache Validation          | 1d               | Safety |

**Key Metrics:**

- Reduce code duplication by 40%
- Improve query performance 3x
- Increase test coverage to 85%
- Reduce unsafe blocks by 90%

**Rustification Checklist:**

- [ ] Replace string-based SQL building with type-safe constructs
- [ ] Implement proper error propagation
- [ ] Use RAII guards for resources
- [ ] Apply Clippy pedantic checks
- [ ] Add documentation for all public interfaces
- [ ] Implement zero-cost abstractions for hot paths

This proposal focuses on structural improvements while maintaining backward compatibility. Each component can be implemented incrementally with careful feature flagging.

Citations: [1] https://ppl-ai-file-upload.s3.amazonaws.com/web/direct-files/3134098/b7b01259-533c-4713-b77a-cf084e861aef/paste.txt
