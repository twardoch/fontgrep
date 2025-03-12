# Fontgrep TODO

## 1. Immediate steps


## 2. Further steps

### 2.1. Memory Optimization

- [ ] Implement streaming for large result sets.
- [ ] Consider lazy loading for large font data.
- [ ] Optimize `FontInfo` size: The `FontInfo` struct currently stores strings for axes, features, scripts, and tables. These could be represented more compactly using indices into shared string tables (similar to how `name` table entries are often handled), particularly if caching is enabled.

### 2.2. Testing

- [ ] Add tests for various edge cases, such as:
  - Fonts with missing tables.
  - Fonts with invalid data in tables.
  - Fonts with very large numbers of glyphs, features, axes, etc.
  - Empty font files.
  - Invalid cache files.
  - Database access conflicts (multiple processes trying to write).

### 2.3. Code Simplification and Removal of Non-Essential Parts (NEW)

- [ ] **Review and remove unused `FontgrepError` variants**: Are all error variants _actually_ used?

### 2.4. Plan of Action (Prioritized)

1.  [ ] **CLI Argument Consolidation:** Explore consolidating CLI argument parsing.
2.  [ ] **Further Performance Optimization:** Profile database queries and optimize with indices as needed.
