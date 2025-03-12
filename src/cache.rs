// this_file: fontgrep/src/cache.rs
//
// Cache implementation for font information

use crate::{
    font::FontInfo,
    query::QueryCriteria,
    Result, DEFAULT_BATCH_SIZE,
};
use rusqlite::{params, Connection, ToSql, OptionalExtension};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Font cache for storing and retrieving font information
pub struct FontCache {
    conn: Option<Arc<Mutex<Connection>>>, // For in-memory databases
    path: PathBuf,
}

impl FontCache {
    /// Create a new font cache
    pub fn new(cache_path: Option<&str>) -> Result<Self> {
        let path = if let Some(path) = cache_path {
            if path == ":memory:" {
                // In-memory database
                let conn = Connection::open_in_memory()?;
                
                // Set pragmas for better performance
                conn.execute_batch("
                    PRAGMA journal_mode = WAL;
                    PRAGMA synchronous = NORMAL;
                    PRAGMA temp_store = MEMORY;
                    PRAGMA mmap_size = 30000000000;
                    PRAGMA page_size = 4096;
                    PRAGMA cache_size = -2000;
                    PRAGMA foreign_keys = ON;
                ")?;
                
                initialize_schema(&conn)?;
                
                return Ok(Self {
                    conn: Some(Arc::new(Mutex::new(conn))),
                    path: PathBuf::from(":memory:"),
                });
            }
            PathBuf::from(path)
        } else {
            crate::utils::determine_cache_path(None)?
        };
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Check if the database file exists
        let db_exists = path.exists();
        
        // Open the database
        let conn = Connection::open(&path)?;
        
        // Set pragmas for better performance - only needed once when creating the database
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 30000000000;
            PRAGMA page_size = 4096;
            PRAGMA cache_size = -2000;
            PRAGMA foreign_keys = ON;
        ")?;
        
        // Initialize schema if the database is new
        if !db_exists {
            initialize_schema(&conn)?;
        }
        
        Ok(Self {
            conn: None,
            path,
        })
    }
    
    /// Get the cache path
    pub fn get_cache_path(&self) -> &PathBuf {
        &self.path
    }
    
    /// Check if a font needs to be updated in the cache
    pub fn needs_update(&self, path: &str, mtime: i64, size: i64) -> Result<bool> {
        let conn = self.get_connection()?;
        
        // Check if the font exists and has the same mtime and size
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM fonts WHERE path = ? AND mtime = ? AND size = ?)",
            params![path, mtime, size],
            |row| row.get(0),
        )?;
        
        Ok(!exists)
    }
    
    /// Update a font in the cache
    pub fn update_font(&self, path: &str, info: &FontInfo, mtime: i64, size: i64) -> Result<()> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;
        let guard = TransactionGuard::new(tx);
        
        // Get or create font_id
        let font_id = {
            // First try to get existing font_id
            let mut stmt = guard.transaction().prepare(
                "SELECT id FROM fonts WHERE path = ?"
            )?;
            
            let font_id: Option<i64> = stmt.query_row(
                params![path],
                |row| row.get(0),
            ).optional()?;
            
            if let Some(id) = font_id {
                // Update existing font
                guard.transaction().execute(
                    "UPDATE fonts SET name = ?, is_variable = ?, mtime = ?, size = ?, charset = ? WHERE id = ?",
                    params![
                        info.name_string,
                        info.is_variable,
                        mtime,
                        size,
                        info.charset_string(),
                        id
                    ],
                )?;
                
                // Clear existing properties
                guard.transaction().execute(
                    "DELETE FROM font_properties WHERE font_id = ?",
                    params![id],
                )?;
                
                id
            } else {
                // Insert new font
                guard.transaction().execute(
                    "INSERT INTO fonts (path, name, is_variable, mtime, size, charset) VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        path,
                        info.name_string,
                        info.is_variable,
                        mtime,
                        size,
                        info.charset_string()
                    ],
                )?;
                
                guard.transaction().last_insert_rowid()
            }
        };
        
        // Insert properties
        self.batch_insert_properties(&guard, font_id, "axis", &info.axes)?;
        self.batch_insert_properties(&guard, font_id, "feature", &info.features)?;
        self.batch_insert_properties(&guard, font_id, "script", &info.scripts)?;
        self.batch_insert_properties(&guard, font_id, "table", &info.tables)?;
        
        guard.commit()?;
        
        Ok(())
    }
    
    /// Batch update fonts in the cache
    pub fn batch_update_fonts(&self, fonts: Vec<(String, FontInfo, i64, i64)>) -> Result<()> {
        if fonts.is_empty() {
            return Ok(());
        }
        
        // Process in batches to avoid excessive memory usage
        let batch_size = DEFAULT_BATCH_SIZE;
        for chunk in fonts.chunks(batch_size) {
            let mut conn = self.get_connection()?;
            let tx = conn.transaction()?;
            let guard = TransactionGuard::new(tx);
            
            {
                let mut font_stmt = guard.transaction().prepare_cached(
                    "INSERT OR REPLACE INTO fonts (path, name, is_variable, mtime, size, charset) VALUES (?, ?, ?, ?, ?, ?)"
                )?;
                
                for (path, info, mtime, size) in chunk {
                    // Insert or replace font
                    font_stmt.execute(params![
                        path,
                        info.name_string,
                        info.is_variable,
                        mtime,
                        size,
                        info.charset_string()
                    ])?;
                    
                    let font_id = guard.transaction().last_insert_rowid();
                    
                    // Clear existing properties
                    guard.transaction().execute(
                        "DELETE FROM font_properties WHERE font_id = ?",
                        params![font_id],
                    )?;
                    
                    // Insert properties
                    self.batch_insert_properties(&guard, font_id, "axis", &info.axes)?;
                    self.batch_insert_properties(&guard, font_id, "feature", &info.features)?;
                    self.batch_insert_properties(&guard, font_id, "script", &info.scripts)?;
                    self.batch_insert_properties(&guard, font_id, "table", &info.tables)?;
                }
            } // font_stmt is dropped here
            
            guard.commit()?;
        }
        
        Ok(())
    }
    
    /// Query fonts based on criteria
    pub fn query(&self, criteria: &QueryCriteria) -> Result<Vec<String>> {
        // Build the query
        let mut builder = QueryBuilder::new();
        
        // Add criteria
        if criteria.variable {
            builder = builder.with_variable();
        }
        
        if !criteria.axes.is_empty() {
            builder = builder.with_property("axis", &criteria.axes);
        }
        
        if !criteria.features.is_empty() {
            builder = builder.with_property("feature", &criteria.features);
        }
        
        if !criteria.scripts.is_empty() {
            builder = builder.with_property("script", &criteria.scripts);
        }
        
        if !criteria.tables.is_empty() {
            builder = builder.with_property("table", &criteria.tables);
        }
        
        if !criteria.name_patterns.is_empty() {
            builder = builder.with_name_patterns(&criteria.name_patterns);
        }
        
        if !criteria.charset.is_empty() {
            builder = builder.with_charset(&criteria.charset);
        }
        
        let (query, params) = builder.build();
        
        // Execute the query with proper parameter conversion
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&query)?;
        
        let params_slice: Vec<&dyn ToSql> = params.iter()
            .map(|p| p.as_ref() as &dyn ToSql)
            .collect();
        
        // Use query_map for more efficient memory usage
        let rows = stmt.query_map(params_slice.as_slice(), |row| row.get::<_, String>(0))?;
        
        // Collect results
        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result?);
        }
        
        Ok(results)
    }
    
    /// Get all font paths in the cache
    pub fn get_all_font_paths(&self) -> Result<Vec<String>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT path FROM fonts")?;
        
        // Use query_map for more efficient memory usage
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        
        // Collect results
        let mut paths = Vec::new();
        for row_result in rows {
            paths.push(row_result?);
        }
        
        Ok(paths)
    }
    
    /// Clean missing fonts from the cache
    pub fn clean_missing_fonts(&self, existing_paths: &HashSet<String>) -> Result<()> {
        let mut conn = self.get_connection()?;
        
        // Get all paths in the cache and collect them first
        let missing_ids = {
            let mut stmt = conn.prepare("SELECT id, path FROM fonts")?;
            let rows = stmt.query_map([], |row| {
                let id: i64 = row.get(0)?;
                let path: String = row.get(1)?;
                Ok((id, path))
            })?;
            
            // Collect all missing IDs
            let mut ids = Vec::new();
            for result in rows {
                let (id, path) = result?;
                if !existing_paths.contains(&path) {
                    ids.push(id);
                }
            }
            ids
        };
        
        // Delete missing fonts
        let tx = conn.transaction()?;
        for id in &missing_ids {
            tx.execute(
                "DELETE FROM font_properties WHERE font_id = ?",
                params![id],
            )?;
            
            tx.execute(
                "DELETE FROM fonts WHERE id = ?",
                params![id],
            )?;
        }
        
        tx.commit()?;
        
        Ok(())
    }
    
    /// Get a connection to the database
    fn get_connection(&self) -> Result<Connection> {
        if let Some(conn) = &self.conn {
            // For in-memory databases, we need to return a connection that shares the same data
            let conn_guard = conn.lock().unwrap();
            let backup_conn = Connection::open_in_memory()?;
            
            // Copy schema and data using SQL
            let tables: Vec<String> = {
                let mut stmt = conn_guard.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
                let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
                rows.collect::<std::result::Result<Vec<String>, _>>()?
            };
            
            for table in &tables {
                // Get table schema
                let schema: String = conn_guard.query_row(
                    "SELECT sql FROM sqlite_master WHERE type='table' AND name=?",
                    params![table],
                    |row| row.get(0),
                )?;
                
                // Create table in backup connection
                backup_conn.execute_batch(&schema)?;
                
                // Copy data
                let rows_data = {
                    let mut stmt = conn_guard.prepare(&format!("SELECT * FROM {}", table))?;
                    let column_count = stmt.column_count();
                    
                    let mut all_rows = Vec::new();
                    let mut rows = stmt.query([])?;
                    
                    while let Some(row) = rows.next()? {
                        let mut values = Vec::new();
                        for i in 0..column_count {
                            let value: String = row.get(i)?;
                            values.push(format!("'{}'", value.replace('\'', "''")));
                        }
                        all_rows.push(values);
                    }
                    all_rows
                };
                
                // Insert the data
                for values in rows_data {
                    let insert_sql = format!(
                        "INSERT INTO {} VALUES ({})",
                        table,
                        values.join(", ")
                    );
                    
                    backup_conn.execute_batch(&insert_sql)?;
                }
            }
            
            Ok(backup_conn)
        } else {
            // For file-based databases, simply open a direct connection
            // No need for in-memory backup
            Ok(Connection::open(&self.path)?)
        }
    }
    
    /// Batch insert properties
    fn batch_insert_properties(
        &self,
        guard: &TransactionGuard,
        font_id: i64,
        prop_type: &str,
        tags: &[String],
    ) -> Result<()> {
        if tags.is_empty() {
            return Ok(());
        }
        
        let mut stmt = guard.transaction().prepare_cached(
            "INSERT INTO font_properties (font_id, type, value) VALUES (?, ?, ?)"
        )?;
        
        for tag in tags {
            stmt.execute(params![font_id, prop_type, tag])?;
        }
        
        Ok(())
    }
}

/// Initialize the database schema
fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        -- Create fonts table
        CREATE TABLE IF NOT EXISTS fonts (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            is_variable INTEGER NOT NULL,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL,
            charset TEXT NOT NULL
        );
        
        -- Create font properties table
        CREATE TABLE IF NOT EXISTS font_properties (
            id INTEGER PRIMARY KEY,
            font_id INTEGER NOT NULL,
            type TEXT NOT NULL,
            value TEXT NOT NULL,
            FOREIGN KEY (font_id) REFERENCES fonts(id) ON DELETE CASCADE
        );
        
        -- Create indices
        CREATE INDEX IF NOT EXISTS idx_fonts_path ON fonts(path);
        CREATE INDEX IF NOT EXISTS idx_fonts_name ON fonts(name);
        CREATE INDEX IF NOT EXISTS idx_fonts_is_variable ON fonts(is_variable);
        CREATE INDEX IF NOT EXISTS idx_font_properties_font_id ON font_properties(font_id);
        CREATE INDEX IF NOT EXISTS idx_font_properties_type ON font_properties(type);
        CREATE INDEX IF NOT EXISTS idx_font_properties_value ON font_properties(value);
        CREATE INDEX IF NOT EXISTS idx_font_properties_type_value ON font_properties(type, value);
    ")?;
    
    Ok(())
}

/// Transaction guard to ensure transactions are rolled back if not committed
struct TransactionGuard<'a> {
    tx: Option<rusqlite::Transaction<'a>>,
}

impl<'a> TransactionGuard<'a> {
    /// Create a new transaction guard
    fn new(tx: rusqlite::Transaction<'a>) -> Self {
        Self { tx: Some(tx) }
    }
    
    /// Commit the transaction
    fn commit(mut self) -> Result<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit()?;
        }
        Ok(())
    }
    
    /// Get the transaction
    fn transaction(&self) -> &rusqlite::Transaction<'a> {
        self.tx.as_ref().unwrap()
    }
}

impl<'a> Drop for TransactionGuard<'a> {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.rollback();
        }
    }
}

/// Query builder for constructing SQL queries
struct QueryBuilder {
    where_clauses: Vec<String>,
    join_clauses: Vec<String>,
    params: Vec<Box<dyn ToSql>>,
    join_counter: usize,
}

impl QueryBuilder {
    /// Create a new query builder
    fn new() -> Self {
        Self {
            where_clauses: Vec::new(),
            join_clauses: Vec::new(),
            params: Vec::new(),
            join_counter: 0,
        }
    }
    
    /// Add variable font criteria
    fn with_variable(mut self) -> Self {
        self.where_clauses.push("f.is_variable = 1".to_string());
        self
    }
    
    /// Add property criteria
    fn with_property(mut self, prop_type: &str, tags: &[String]) -> Self {
        if tags.is_empty() {
            return self;
        }
        
        self.join_counter += 1;
        let alias = format!("p{}", self.join_counter);
        
        // Join with font_properties table
        self.join_clauses.push(format!(
            "JOIN font_properties {} ON {}.font_id = f.id AND {}.type = ?",
            alias, alias, alias
        ));
        self.params.push(Box::new(prop_type.to_string()));
        
        // Add WHERE clause for property values
        if tags.len() == 1 {
            // Optimize for the common case of a single tag
            self.where_clauses.push(format!("{}.value = ?", alias));
            self.params.push(Box::new(tags[0].clone()));
        } else {
            // Use IN clause for multiple tags
            let placeholders = (0..tags.len()).map(|_| "?").collect::<Vec<_>>().join(", ");
            self.where_clauses.push(format!("{}.value IN ({})", alias, placeholders));
            
            // Add parameters
            for tag in tags {
                self.params.push(Box::new(tag.clone()));
            }
        }
        
        self
    }
    
    /// Add name pattern criteria
    fn with_name_patterns(mut self, patterns: &[String]) -> Self {
        if patterns.is_empty() {
            return self;
        }
        
        // For each pattern, create a condition that tries to match it as a regex-like pattern
        let mut conditions = Vec::new();
        
        for pattern in patterns {
            // Convert regex-like pattern to SQL LIKE pattern
            // This is a simplified conversion that handles common cases
            let sql_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
                // Exact match: ^pattern$
                let inner = &pattern[1..pattern.len()-1];
                format!("f.name = '{}'", inner.replace('\'', "''"))
            } else if pattern.starts_with('^') {
                // Starts with: ^pattern
                let inner = &pattern[1..];
                format!("f.name LIKE '{}%'", inner.replace('\'', "''"))
            } else if pattern.ends_with('$') {
                // Ends with: pattern$
                let inner = &pattern[..pattern.len()-1];
                format!("f.name LIKE '%{}'", inner.replace('\'', "''"))
            } else {
                // Contains: pattern
                format!("f.name LIKE '%{}%'", pattern.replace('\'', "''"))
            };
            
            conditions.push(sql_pattern);
        }
        
        // Join all conditions with OR
        if !conditions.is_empty() {
            self.where_clauses.push(format!("({})", conditions.join(" OR ")));
        }
        
        self
    }
    
    /// Add charset criteria
    fn with_charset(mut self, charset: &str) -> Self {
        if charset.is_empty() {
            return self;
        }
        
        // Check for each character individually
        let chars: Vec<char> = charset.chars().collect();
        
        if chars.len() == 1 {
            // Optimize for the common case of a single character
            // Use direct comparison instead of LIKE for better accuracy
            self.where_clauses.push("f.charset LIKE ?".to_string());
            // Escape special characters in the LIKE pattern
            let escaped_char = escape_like_pattern(&chars[0].to_string());
            self.params.push(Box::new(format!("%{}%", escaped_char)));
        } else {
            // For multiple characters, check that each one is present
            let conditions = chars.iter()
                .map(|_| "f.charset LIKE ?")
                .collect::<Vec<_>>()
                .join(" AND ");
            
            self.where_clauses.push(format!("({})", conditions));
            
            // Add parameters for each character with proper escaping
            for &c in &chars {
                // Escape special characters in the LIKE pattern
                let escaped_char = escape_like_pattern(&c.to_string());
                self.params.push(Box::new(format!("%{}%", escaped_char)));
            }
        }
        
        self
    }
    
    /// Build the query
    fn build(self) -> (String, Vec<Box<dyn ToSql>>) {
        // Start with the basic SELECT and FROM clauses
        let mut query = "SELECT DISTINCT f.path FROM fonts f".to_string();
        
        // Add JOIN clauses
        for join in self.join_clauses {
            query = format!("{} {}", query, join);
        }
        
        // Add WHERE clauses
        if !self.where_clauses.is_empty() {
            query = format!("{} WHERE {}", query, self.where_clauses.join(" AND "));
        }
        
        (query, self.params)
    }
}

/// Escape special characters in a LIKE pattern
fn escape_like_pattern(s: &str) -> String {
    // Escape special characters: % _ [ ] ^
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        if c == '%' || c == '_' || c == '[' || c == ']' || c == '^' {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_builder() {
        let builder = QueryBuilder::new()
            .with_variable()
            .with_property("axis", &["wght".to_string(), "wdth".to_string()])
            .with_name_patterns(&["Roboto".to_string()]);
        
        let (query, params) = builder.build();
        
        assert!(query.contains("SELECT DISTINCT"));
        assert!(query.contains("f.is_variable = 1"));
        assert!(query.contains("JOIN font_properties"));
        assert!(query.contains("f.name LIKE"));
        assert_eq!(params.len(), 3); // 1 for type, 2 for values
    }
    
    #[test]
    fn test_escape_like_pattern() {
        assert_eq!(escape_like_pattern("abc"), "abc");
        assert_eq!(escape_like_pattern("a%c"), "a\\%c");
        assert_eq!(escape_like_pattern("a_c"), "a\\_c");
        assert_eq!(escape_like_pattern("a[c"), "a\\[c");
        assert_eq!(escape_like_pattern("a]c"), "a\\]c");
        assert_eq!(escape_like_pattern("a^c"), "a\\^c");
        assert_eq!(escape_like_pattern("a%_[]]^c"), "a\\%\\_\\[\\]\\]\\^c");
    }
}