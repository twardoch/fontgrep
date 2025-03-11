// this_file: fontgrep/src/cache.rs

use crate::fontinfo::FontInfo;
use dirs::data_dir;
use rusqlite::{params, Connection, Result as SqlResult, ToSql};
use skrifa::Tag;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

/// Represents the cache for font metadata
#[derive(Clone)]
pub struct FontCache {
    conn: Arc<Mutex<Connection>>,
}

impl FontCache {
    /// Creates a new cache or opens an existing one
    pub fn new(cache_path: Option<&str>) -> SqlResult<Self> {
        let path = determine_cache_path(cache_path)?;
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better performance
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
        ")?;
        
        // Initialize the database schema if it doesn't exist
        // Use INTEGER PRIMARY KEY for font_id to get auto-incrementing rowid
        conn.execute(
            "CREATE TABLE IF NOT EXISTS fonts (
                font_id INTEGER PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                mtime INTEGER NOT NULL,
                size INTEGER NOT NULL,
                is_variable BOOLEAN NOT NULL,
                name_string TEXT,
                charset_string TEXT
            )",
            [],
        )?;
        
        // Use font_id as foreign key to reduce storage size
        conn.execute(
            "CREATE TABLE IF NOT EXISTS axes (
                font_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (font_id, tag),
                FOREIGN KEY (font_id) REFERENCES fonts(font_id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS features (
                font_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (font_id, tag),
                FOREIGN KEY (font_id) REFERENCES fonts(font_id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS scripts (
                font_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (font_id, tag),
                FOREIGN KEY (font_id) REFERENCES fonts(font_id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tables (
                font_id INTEGER NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (font_id, tag),
                FOREIGN KEY (font_id) REFERENCES fonts(font_id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create indices for better query performance
        conn.execute_batch("
            CREATE INDEX IF NOT EXISTS idx_fonts_path ON fonts(path);
            CREATE INDEX IF NOT EXISTS idx_axes_tag ON axes(tag);
            CREATE INDEX IF NOT EXISTS idx_features_tag ON features(tag);
            CREATE INDEX IF NOT EXISTS idx_scripts_tag ON scripts(tag);
            CREATE INDEX IF NOT EXISTS idx_tables_tag ON tables(tag);
        ")?;
        
        Ok(FontCache { conn: Arc::new(Mutex::new(conn)) })
    }
    
    /// Checks if a font needs to be updated in the cache
    pub fn needs_update(&self, path: &str, mtime: i64, size: i64) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT mtime, size FROM fonts WHERE path = ?1"
        )?;
        
        let rows = stmt.query_map([path], |row| {
            let cached_mtime: i64 = row.get(0)?;
            let cached_size: i64 = row.get(1)?;
            Ok((cached_mtime, cached_size))
        })?;
        
        for result in rows {
            let (cached_mtime, cached_size) = result?;
            // If either mtime or size has changed, the font needs updating
            return Ok(cached_mtime != mtime || cached_size != size);
        }
        
        // Font not in cache, so it needs to be added
        Ok(true)
    }
    
    /// Gets the font_id for a path, creating a new entry if needed
    #[allow(dead_code)]
    pub fn get_font_id(&self, path: &str, mtime: i64, size: i64) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        // First try to get existing font_id
        let mut stmt = conn.prepare(
            "SELECT font_id FROM fonts WHERE path = ?1"
        )?;
        
        let rows = stmt.query_map([path], |row| {
            row.get(0)
        })?;
        
        for result in rows {
            return result;
        }
        
        // Font not found, insert a new entry
        conn.execute(
            "INSERT INTO fonts (path, mtime, size, is_variable, name_string, charset_string) 
             VALUES (?1, ?2, ?3, 0, '', '')",
            params![path, mtime, size],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    /// Updates a font in the cache with the extracted information
    pub fn update_font(&mut self, path: &str, font_info: &FontInfo, mtime: i64, size: i64) -> SqlResult<()> {
        let mut conn = self.conn.lock().unwrap();
        // Begin transaction for better performance
        let tx = conn.transaction()?;
        
        // Get or create font_id
        let font_id = {
            // First try to get existing font_id
            let mut stmt = tx.prepare(
                "SELECT font_id FROM fonts WHERE path = ?1"
            )?;
            
            let rows = stmt.query_map([path], |row| {
                row.get(0)
            })?;
            
            let mut existing_id = None;
            for result in rows {
                existing_id = Some(result?);
                break;
            }
            
            match existing_id {
                Some(id) => id,
                None => {
                    // Font not found, insert a new entry
                    tx.execute(
                        "INSERT INTO fonts (path, mtime, size, is_variable, name_string, charset_string) 
                         VALUES (?1, ?2, ?3, 0, '', '')",
                        params![path, mtime, size],
                    )?;
                    
                    tx.last_insert_rowid()
                }
            }
        };
        
        // Update the main font record
        tx.execute(
            "UPDATE fonts SET 
                mtime = ?1, 
                size = ?2, 
                is_variable = ?3, 
                name_string = ?4, 
                charset_string = ?5 
             WHERE font_id = ?6",
            params![
                mtime, 
                size, 
                font_info.is_variable, 
                &font_info.name_string, 
                &font_info.charset_string, 
                font_id
            ],
        )?;
        
        // Clear existing related records
        tx.execute("DELETE FROM axes WHERE font_id = ?1", [font_id])?;
        tx.execute("DELETE FROM features WHERE font_id = ?1", [font_id])?;
        tx.execute("DELETE FROM scripts WHERE font_id = ?1", [font_id])?;
        tx.execute("DELETE FROM tables WHERE font_id = ?1", [font_id])?;
        
        // Insert axes
        {
            let mut stmt = tx.prepare(
                "INSERT INTO axes (font_id, tag) VALUES (?1, ?2)"
            )?;
            for axis in &font_info.axes {
                stmt.execute(params![font_id, axis.to_string()])?;
            }
        }
        
        // Insert features
        {
            let mut stmt = tx.prepare(
                "INSERT INTO features (font_id, tag) VALUES (?1, ?2)"
            )?;
            for feature in &font_info.features {
                stmt.execute(params![font_id, feature])?;
            }
        }
        
        // Insert scripts
        {
            let mut stmt = tx.prepare(
                "INSERT INTO scripts (font_id, tag) VALUES (?1, ?2)"
            )?;
            for script in &font_info.scripts {
                stmt.execute(params![font_id, script])?;
            }
        }
        
        // Insert tables
        {
            let mut stmt = tx.prepare(
                "INSERT INTO tables (font_id, tag) VALUES (?1, ?2)"
            )?;
            for table in &font_info.tables {
                stmt.execute(params![font_id, table.to_string()])?;
            }
        }
        
        // Commit the transaction
        tx.commit()?;
        
        Ok(())
    }
    
    /// Removes fonts from the cache that no longer exist
    pub fn clean_missing_fonts(&mut self, existing_paths: &HashSet<String>) -> SqlResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let mut to_delete = Vec::new();
        
        {
            let mut stmt = conn.prepare("SELECT path FROM fonts")?;
            let rows = stmt.query_map([], |row| {
                let path: String = row.get(0)?;
                Ok(path)
            })?;
            
            for result in rows {
                let path = result?;
                if !existing_paths.contains(&path) {
                    to_delete.push(path);
                }
            }
        }
        
        let tx = conn.transaction()?;
        for path in to_delete {
            tx.execute("DELETE FROM fonts WHERE path = ?1", [&path])?;
        }
        tx.commit()?;
        
        Ok(())
    }
    
    /// Queries the cache for fonts matching the given criteria
    #[allow(dead_code)]
    pub fn query(
        &self,
        axes: &[String],
        features: &[String],
        scripts: &[String],
        tables: &[Tag],
        name_regexes: &[String],
        variable: bool,
        charset_query: Option<&str>,
    ) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            "SELECT DISTINCT f.path FROM fonts f "
        );
        
        let mut where_clauses = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();
        
        // Variable font filter
        if variable {
            where_clauses.push("f.is_variable = 1".to_string());
        }
        
        // Axis filters
        if !axes.is_empty() {
            for (i, axis) in axes.iter().enumerate() {
                let join_clause = format!("JOIN axes a{} ON f.font_id = a{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("a{}.tag = ?", i + 1);
                where_clauses.push(where_clause);
                params.push(Box::new(axis.clone()));
            }
        }
        
        // Feature filters
        if !features.is_empty() {
            for (i, feature) in features.iter().enumerate() {
                let join_clause = format!("JOIN features ft{} ON f.font_id = ft{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("ft{}.tag = ?", i + 1);
                where_clauses.push(where_clause);
                params.push(Box::new(feature.clone()));
            }
        }
        
        // Script filters
        if !scripts.is_empty() {
            for (i, script) in scripts.iter().enumerate() {
                let join_clause = format!("JOIN scripts s{} ON f.font_id = s{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("s{}.tag = ?", i + 1);
                where_clauses.push(where_clause);
                params.push(Box::new(script.clone()));
            }
        }
        
        // Table filters
        if !tables.is_empty() {
            for (i, table) in tables.iter().enumerate() {
                let join_clause = format!("JOIN tables t{} ON f.font_id = t{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("t{}.tag = ?", i + 1);
                where_clauses.push(where_clause);
                params.push(Box::new(table.to_string()));
            }
        }
        
        // Name regex filters
        if !name_regexes.is_empty() {
            for regex in name_regexes {
                let where_clause = "f.name_string REGEXP ?".to_string();
                where_clauses.push(where_clause);
                params.push(Box::new(regex.clone()));
            }
        }
        
        // Charset query filter
        if let Some(charset) = charset_query {
            // If the charset is short (likely a text query), require ALL characters
            if charset.len() <= 10 {
                for ch in charset.chars() {
                    let where_clause = "f.charset_string LIKE ?".to_string();
                    where_clauses.push(where_clause);
                    params.push(Box::new(format!("%{}%", ch)));
                }
            } else {
                // For longer charsets (likely a range query), just check for a few characters
                let sample: String = charset.chars().take(5).collect();
                let where_clause = "f.charset_string LIKE ?".to_string();
                where_clauses.push(where_clause);
                params.push(Box::new(format!("%{}%", sample)));
            }
        }
        
        // Add WHERE clause if we have any conditions
        if !where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&where_clauses.join(" AND "));
        }
        
        // Execute the query
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |row| {
            let path: String = row.get(0)?;
            Ok(path)
        })?;
        
        let mut results = Vec::new();
        for result in rows {
            results.push(result?);
        }
        
        Ok(results)
    }
    
    /// Checks if a specific font matches the given criteria
    pub fn font_matches(
        &self,
        path: &str,
        axes: &[String],
        features: &[String],
        scripts: &[String],
        tables: &[Tag],
        name_regexes: &[String],
        variable: bool,
        charset_query: Option<&str>,
    ) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            "SELECT 1 FROM fonts f "
        );
        
        let mut where_clauses = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();
        
        // Path filter
        where_clauses.push("f.path = ?".to_string());
        params.push(Box::new(path.to_string()));
        
        // Variable font filter
        if variable {
            where_clauses.push("f.is_variable = 1".to_string());
        }
        
        // Axis filters
        if !axes.is_empty() {
            for (i, axis) in axes.iter().enumerate() {
                let join_clause = format!("JOIN axes a{} ON f.font_id = a{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("a{}.tag = ?", i + 2);
                where_clauses.push(where_clause);
                params.push(Box::new(axis.clone()));
            }
        }
        
        // Feature filters
        if !features.is_empty() {
            for (i, feature) in features.iter().enumerate() {
                let join_clause = format!("JOIN features ft{} ON f.font_id = ft{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("ft{}.tag = ?", i + 2 + axes.len());
                where_clauses.push(where_clause);
                params.push(Box::new(feature.clone()));
            }
        }
        
        // Script filters
        if !scripts.is_empty() {
            for (i, script) in scripts.iter().enumerate() {
                let join_clause = format!("JOIN scripts s{} ON f.font_id = s{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("s{}.tag = ?", i + 2 + axes.len() + features.len());
                where_clauses.push(where_clause);
                params.push(Box::new(script.clone()));
            }
        }
        
        // Table filters
        if !tables.is_empty() {
            for (i, table) in tables.iter().enumerate() {
                let join_clause = format!("JOIN tables t{} ON f.font_id = t{}.font_id ", i, i);
                query.push_str(&join_clause);
                
                let where_clause = format!("t{}.tag = ?", i + 2 + axes.len() + features.len() + scripts.len());
                where_clauses.push(where_clause);
                params.push(Box::new(table.to_string()));
            }
        }
        
        // Name regex filters
        if !name_regexes.is_empty() {
            for regex in name_regexes {
                let where_clause = "f.name_string REGEXP ?".to_string();
                where_clauses.push(where_clause);
                params.push(Box::new(regex.clone()));
            }
        }
        
        // Charset query filter
        if let Some(charset) = charset_query {
            // If the charset is short (likely a text query), require ALL characters
            if charset.len() <= 10 {
                for ch in charset.chars() {
                    let where_clause = "f.charset_string LIKE ?".to_string();
                    where_clauses.push(where_clause);
                    params.push(Box::new(format!("%{}%", ch)));
                }
            } else {
                // For longer charsets (likely a range query), just check for a few characters
                let sample: String = charset.chars().take(5).collect();
                let where_clause = "f.charset_string LIKE ?".to_string();
                where_clauses.push(where_clause);
                params.push(Box::new(format!("%{}%", sample)));
            }
        }
        
        // Add WHERE clause
        query.push_str(" WHERE ");
        query.push_str(&where_clauses.join(" AND "));
        query.push_str(" LIMIT 1");
        
        // Execute the query
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |_| {
            Ok(true)
        })?;
        
        for result in rows {
            return Ok(result?);
        }
        
        Ok(false)
    }
}

/// Determines the path to use for the cache file
fn determine_cache_path(cache_path: Option<&str>) -> SqlResult<PathBuf> {
    match cache_path {
        Some("default") | None => {
            // Use the default cache location in the user's data directory
            let data_dir = data_dir().ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("Could not determine data directory".to_string())
            })?;
            
            let fontgrep_dir = data_dir.join("fontgrep");
            fs::create_dir_all(&fontgrep_dir).map_err(|e| {
                rusqlite::Error::InvalidParameterName(format!("Could not create cache directory: {}", e))
            })?;
            
            Ok(fontgrep_dir.join("fontcache.db"))
        }
        Some(path) => {
            // Use the specified path
            let path = PathBuf::from(path);
            
            // If it's a directory, create it and use a default filename
            if path.is_dir() || (!path.exists() && path.to_string_lossy().ends_with('/')) {
                fs::create_dir_all(&path).map_err(|e| {
                    rusqlite::Error::InvalidParameterName(format!("Could not create cache directory: {}", e))
                })?;
                
                Ok(path.join("fontcache.db"))
            } else {
                // It's a file path, ensure the parent directory exists
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| {
                            rusqlite::Error::InvalidParameterName(format!("Could not create parent directory: {}", e))
                        })?;
                    }
                }
                
                Ok(path)
            }
        }
    }
}

/// Gets the modification time of a file as seconds since the epoch
pub fn get_file_mtime(path: &Path) -> std::io::Result<i64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata.modified()?;
    let duration = mtime.duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    Ok(duration.as_secs() as i64)
}

/// Gets the size of a file in bytes
pub fn get_file_size(path: &Path) -> std::io::Result<i64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len() as i64)
}