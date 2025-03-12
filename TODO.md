# Fontgrep TODO


## 1. Performance Optimization

- [ ] **Optimize charset generation**
  - Replace the linear scan with direct charmap iteration:

```rust
pub fn create_charset(font: &FontRef) -> BTreeSet<u32> {
    font.charmap().iter()
        .filter(|(cp, _)| !is_invalid_unicode(*cp))
        .map(|(cp, _)| cp)
        .collect()
}
```

- [ ] **Improve database operations**
  - Use prepared statements for repetitive operations
  - Add appropriate indexes for common query patterns
  - Implement transaction batching for bulk operations

## 2. Error Handling

- [ ] **Audit and replace remaining `unwrap()`/`expect()` calls**
  - Identify all instances in the codebase
  - Replace with proper error propagation using `?` operator
  - Ensure meaningful error messages

- [ ] **Improve error handling for font parsing**
  - Add better error messages for common font parsing errors
  - Handle TTC (TrueType Collection) files properly
  - Add graceful fallbacks for fonts with minor corruption

## 3. Testing

- [ ] **Add comprehensive tests**
  - Add unit tests for core functionality
  - Add integration tests for end-to-end workflows
  - Test with real font files

## 4. Documentation

- [ ] **Improve documentation**
  - Update README with new command names and examples
  - Document CLI options more thoroughly
  - Add more doc comments to public API

## 5. Memory Optimization

- [ ] **Reduce memory usage**
  - Implement streaming for large result sets
  - Consider lazy loading for large font data

## 6. Implementation Priority

1. ~~CLI improvements~~ (completed)
2. Error handling improvements (highest priority)
3. Performance optimizations for charset generation
4. Database operation improvements
5. Testing additions
6. Documentation improvements
7. Memory optimizations

These focused improvements will significantly enhance the reliability, performance, and usability of fontgrep while maintaining the current functionality.
