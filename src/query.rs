// this_file: fontgrep/src/query.rs
//
// Query execution and font matching

use crate::{
    Result, FontgrepError,
    font::{FontInfo, is_font_file},
    cache::FontCache,
    utils::{get_file_mtime, get_file_size},
};
use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};
use walkdir::WalkDir;

/// Criteria for querying fonts
#[derive(Debug, Clone, Default)]
pub struct QueryCriteria {
    /// Variation axes to search for
    pub axes: Vec<String>,
    
    /// Unicode codepoints to search for
    pub codepoints: Vec<char>,
    
    /// OpenType features to search for
    pub features: Vec<String>,
    
    /// OpenType scripts to search for
    pub scripts: Vec<String>,
    
    /// Font tables to search for
    pub tables: Vec<String>,
    
    /// Regular expressions to match against font names
    pub name_patterns: Vec<String>,
    
    /// Only show variable fonts
    pub variable: bool,
    
    /// Charset string for searching
    pub charset: String,
}

impl QueryCriteria {
    /// Create a new query criteria
    pub fn new(
        axes: Vec<String>,
        codepoints: Vec<char>,
        features: Vec<String>,
        scripts: Vec<String>,
        tables: Vec<String>,
        name_patterns: Vec<String>,
        variable: bool,
    ) -> Self {
        // Convert codepoints to charset string
        let charset = if !codepoints.is_empty() {
            // Create a string from the codepoints directly
            let mut charset_string = String::with_capacity(codepoints.len());
            for cp in &codepoints {
                charset_string.push(*cp);
            }
            charset_string
        } else {
            String::new()
        };
        
        Self {
            axes,
            codepoints,
            features,
            scripts,
            tables,
            name_patterns,
            variable,
            charset,
        }
    }
    
    /// Check if the criteria is empty (no filters)
    pub fn is_empty(&self) -> bool {
        self.axes.is_empty() &&
        self.codepoints.is_empty() &&
        self.features.is_empty() &&
        self.scripts.is_empty() &&
        self.tables.is_empty() &&
        self.name_patterns.is_empty() &&
        !self.variable
    }
    
    /// Get the charset query string if codepoints are specified
    pub fn get_charset_query(&self) -> Option<String> {
        if self.codepoints.is_empty() {
            None
        } else {
            // Return the pre-computed charset string
            Some(self.charset.clone())
        }
    }
}

/// Font query for executing searches
pub struct FontQuery {
    /// Criteria for the query
    criteria: QueryCriteria,
    
    /// Whether to use the cache
    use_cache: bool,
    
    /// Cache for font metadata
    cache: Option<FontCache>,
    
    /// Number of parallel jobs to use
    jobs: usize,
    
    /// Compiled name regexes
    name_regexes: Vec<Regex>,
}

impl FontQuery {
    /// Create a new font query
    pub fn new(
        criteria: QueryCriteria,
        use_cache: bool,
        cache_path: Option<&str>,
        jobs: usize,
    ) -> Self {
        // Compile name regexes with case insensitivity and ensuring patterns match anywhere in the string
        let name_regexes = criteria.name_patterns.iter()
            .filter_map(|pattern| {
                // If the pattern doesn't have explicit anchors, make it match anywhere
                let regex_pattern = if !pattern.starts_with('^') && !pattern.ends_with('$') {
                    format!("(?i).*{}.*", regex::escape(pattern))
                } else {
                    // If it has anchors, just make it case insensitive
                    format!("(?i){}", pattern)
                };
                
                Regex::new(&regex_pattern).ok()
            })
            .collect();
        
        // Initialize cache if needed
        let cache = if use_cache {
            match FontCache::new(cache_path) {
                Ok(cache) => Some(cache),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize cache: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        Self {
            criteria,
            use_cache,
            cache,
            jobs,
            name_regexes,
        }
    }
    
    /// Execute the query
    pub fn execute(&self, paths: &[PathBuf]) -> Result<Vec<String>> {
        // If we're using the cache, try to query it first
        if self.use_cache && self.cache.is_some() {
            match self.query_cache(paths) {
                Ok(results) => return Ok(results),
                Err(e) => {
                    eprintln!("Warning: Cache query failed: {}", e);
                    eprintln!("Falling back to direct directory search");
                }
            }
        }
        
        // If cache query failed or we're not using the cache, search directories directly
        self.search_directories(paths)
    }
    
    /// Query the cache
    fn query_cache(&self, paths: &[PathBuf]) -> Result<Vec<String>> {
        let _cache = self.cache.as_ref().ok_or_else(|| {
            FontgrepError::Cache("Cache not initialized".to_string())
        })?;
        
        // If paths is empty, query all fonts in the cache
        if paths.is_empty() {
            return self.query_cache_all();
        }
        
        // Otherwise, filter by paths
        self.query_cache_filtered(paths)
    }
    
    /// Query all fonts in the cache
    fn query_cache_all(&self) -> Result<Vec<String>> {
        let cache = self.cache.as_ref().ok_or_else(|| {
            FontgrepError::Cache("Cache not initialized".to_string())
        })?;
        
        // If criteria is empty, return all fonts
        if self.criteria.is_empty() {
            return Ok(cache.get_all_font_paths()?);
        }
        
        // Otherwise, query with criteria
        Ok(cache.query(&self.criteria)?)
    }
    
    /// Query the cache with path filtering
    fn query_cache_filtered(&self, paths: &[PathBuf]) -> Result<Vec<String>> {
        let cache = self.cache.as_ref().ok_or_else(|| {
            FontgrepError::Cache("Cache not initialized".to_string())
        })?;
        
        // Get all matching fonts from the cache
        let all_matches = cache.query(&self.criteria)?;
        
        // Filter by paths
        let mut results = Vec::new();
        for path_str in all_matches {
            let path = Path::new(&path_str);
            
            // Check if the path is within any of the specified directories
            for dir in paths {
                if path.starts_with(dir) {
                    results.push(path_str.clone());
                    break;
                }
            }
        }
        
        Ok(results)
    }
    
    /// Search directories directly
    fn search_directories(&self, paths: &[PathBuf]) -> Result<Vec<String>> {
        // Collect all font files from the specified paths
        let font_files = self.collect_font_files(paths)?;
        
        // Process font files in parallel
        let matching_fonts = Arc::new(Mutex::new(Vec::new()));
        
        // Create a channel for immediate printing
        let (tx, rx) = mpsc::channel();
        
        // Spawn a thread to print results as they come in
        let printer_thread = thread::spawn(move || {
            for path in rx {
                println!("{}", path);
            }
        });
        
        // Configure thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .build_global()
            .unwrap_or_default();
        
        // Process files in parallel
        font_files.par_iter().for_each(|path| {
            match self.process_font_file(path) {
                Ok(true) => {
                    // Font matches criteria
                    let path_str = path.to_string_lossy().to_string();
                    
                    // Send to printer thread for immediate output
                    let _ = tx.send(path_str.clone());
                    
                    // Also collect for return value
                    let mut fonts = matching_fonts.lock().unwrap();
                    fonts.push(path_str);
                },
                Ok(false) => {
                    // Font doesn't match criteria
                },
                Err(e) => {
                    eprintln!("Error processing font {}: {}", path.display(), e);
                }
            }
        });
        
        // Drop the sender to signal the printer thread to exit
        drop(tx);
        
        // Wait for printer thread to finish
        let _ = printer_thread.join();
        
        // Return the matching fonts
        let result = matching_fonts.lock().unwrap().clone();
        Ok(result)
    }
    
    /// Collect all font files from the specified paths
    fn collect_font_files(&self, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
        let mut font_files = Vec::new();
        
        for path in paths {
            if path.is_file() {
                // If it's a file, check if it's a font file
                if is_font_file(path) {
                    font_files.push(path.clone());
                }
            } else if path.is_dir() {
                // If it's a directory, walk it recursively
                for entry in WalkDir::new(path).follow_links(true) {
                    match entry {
                        Ok(entry) => {
                            let entry_path = entry.path();
                            if entry_path.is_file() && is_font_file(entry_path) {
                                font_files.push(entry_path.to_path_buf());
                            }
                        },
                        Err(e) => {
                            eprintln!("Error walking directory {}: {}", path.display(), e);
                        }
                    }
                }
            } else {
                eprintln!("Warning: Path does not exist: {}", path.display());
            }
        }
        
        Ok(font_files)
    }
    
    /// Process a font file
    fn process_font_file(&self, path: &Path) -> Result<bool> {
        // Load font info
        let font_info = FontInfo::load(path)?;
        
        // Check if the font matches the criteria
        self.font_matches(&font_info)
    }
    
    /// Check if a font matches the criteria
    fn font_matches(&self, font_info: &FontInfo) -> Result<bool> {
        // Create matchers for each criteria
        let _matches = true;
        
        // Check variable font
        if self.criteria.variable && !font_info.is_variable {
            return Ok(false);
        }
        
        // Check axes
        if !self.criteria.axes.is_empty() {
            let all_axes_match = self.criteria.axes.iter()
                .all(|axis| font_info.axes.contains(axis));
            if !all_axes_match {
                return Ok(false);
            }
        }
        
        // Check features
        if !self.criteria.features.is_empty() {
            let all_features_match = self.criteria.features.iter()
                .all(|feature| font_info.features.contains(feature));
            if !all_features_match {
                return Ok(false);
            }
        }
        
        // Check scripts
        if !self.criteria.scripts.is_empty() {
            let all_scripts_match = self.criteria.scripts.iter()
                .all(|script| font_info.scripts.contains(script));
            if !all_scripts_match {
                return Ok(false);
            }
        }
        
        // Check tables
        if !self.criteria.tables.is_empty() {
            let all_tables_match = self.criteria.tables.iter()
                .all(|table| font_info.tables.contains(&table.to_string()));
            if !all_tables_match {
                return Ok(false);
            }
        }
        
        // Check codepoints
        if !self.criteria.codepoints.is_empty() {
            let charset = font_info.charset_string();
            let all_codepoints_match = self.criteria.codepoints.iter()
                .all(|cp| charset.contains(*cp));
            if !all_codepoints_match {
                return Ok(false);
            }
        }
        
        // Check name patterns
        if !self.name_regexes.is_empty() {
            let any_name_matches = self.name_regexes.iter()
                .any(|pattern| pattern.is_match(&font_info.name_string));
            if !any_name_matches {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Update the cache with fonts from the specified paths
    pub fn update_cache(&self, paths: &[PathBuf], force: bool) -> Result<()> {
        let cache = self.cache.as_ref().ok_or_else(|| {
            FontgrepError::Cache("Cache not initialized".to_string())
        })?;
        
        // Collect all font files from the specified paths
        let font_files = self.collect_font_files(paths)?;
        
        // Process font files in batches
        let _processed = 0;
        let _total = font_files.len();
        
        // Configure thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .build_global()
            .unwrap_or_default();
        
        // Process files in parallel and collect updates
        let updates = Arc::new(Mutex::new(Vec::new()));
        
        font_files.par_iter().for_each(|path| {
            // Get file metadata
            let mtime = match get_file_mtime(path) {
                Ok(mtime) => mtime,
                Err(e) => {
                    eprintln!("Error getting mtime for {}: {}", path.display(), e);
                    return;
                }
            };
            
            let size = match get_file_size(path) {
                Ok(size) => size,
                Err(e) => {
                    eprintln!("Error getting size for {}: {}", path.display(), e);
                    return;
                }
            };
            
            // Check if the font needs updating
            let path_str = path.to_string_lossy().to_string();
            let needs_update = force || match cache.needs_update(&path_str, mtime, size) {
                Ok(needs_update) => needs_update,
                Err(e) => {
                    eprintln!("Error checking if font needs update: {}", e);
                    true // Update anyway if we can't check
                }
            };
            
            if needs_update {
                // Load font info
                match FontInfo::load(path) {
                    Ok(font_info) => {
                        // Print the path being saved
                        println!("{}", path.display());
                        
                        // Add to updates
                        let mut updates_guard = updates.lock().unwrap();
                        updates_guard.push((path_str, font_info, mtime, size));
                    },
                    Err(e) => {
                        eprintln!("Error loading font {}: {}", path.display(), e);
                    }
                }
            }
        });
        
        // Get all updates
        let all_updates = updates.lock().unwrap().clone();
        
        // Update the cache in batches
        cache.batch_update_fonts(all_updates)?;
        
        // Clean up missing fonts
        let existing_paths: HashSet<String> = font_files.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        
        cache.clean_missing_fonts(&existing_paths)?;
        
        Ok(())
    }
    
    /// List all fonts in the cache
    pub fn list_all_fonts(&self) -> Result<Vec<String>> {
        let cache = self.cache.as_ref().ok_or_else(|| {
            FontgrepError::Cache("Cache not initialized".to_string())
        })?;
        
        Ok(cache.get_all_font_paths()?)
    }
    
    /// Clean the cache by removing missing fonts
    pub fn clean_cache(&self) -> Result<()> {
        let cache = self.cache.as_ref().ok_or_else(|| {
            FontgrepError::Cache("Cache not initialized".to_string())
        })?;
        
        // Get all font paths from the cache
        let all_paths = cache.get_all_font_paths()?;
        
        // Check which paths still exist
        let mut existing_paths = HashSet::new();
        for path_str in all_paths {
            let path = Path::new(&path_str);
            if path.exists() {
                existing_paths.insert(path_str);
            }
        }
        
        // Clean up missing fonts
        cache.clean_missing_fonts(&existing_paths)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_criteria_is_empty() {
        let empty = QueryCriteria::default();
        assert!(empty.is_empty());
        
        let with_axes = QueryCriteria {
            axes: vec!["wght".to_string()],
            ..Default::default()
        };
        assert!(!with_axes.is_empty());
        
        let with_variable = QueryCriteria {
            variable: true,
            ..Default::default()
        };
        assert!(!with_variable.is_empty());
    }
    
    #[test]
    fn test_get_charset_query() {
        let empty = QueryCriteria::default();
        assert_eq!(empty.get_charset_query(), None);
        
        let with_codepoints = QueryCriteria {
            codepoints: vec!['A', 'B', 'C'],
            charset: "ABC".to_string(),
            ..Default::default()
        };
        assert_eq!(with_codepoints.get_charset_query(), Some("ABC".to_string()));
    }
    
    #[test]
    fn test_name_pattern_matching() {
        // Create a simple struct that implements the necessary traits for testing
        struct MockFontInfo {
            name_string: String,
        }
        
        impl MockFontInfo {
            fn name_string(&self) -> &str {
                &self.name_string
            }
            
            fn is_variable(&self) -> bool {
                false
            }
            
            fn axes(&self) -> &[String] {
                &[]
            }
            
            fn features(&self) -> &[String] {
                &[]
            }
            
            fn scripts(&self) -> &[String] {
                &[]
            }
            
            fn tables(&self) -> &[String] {
                &[]
            }
            
            fn charset_string(&self) -> &str {
                ""
            }
        }
        
        // Create a mock font info
        let font_info = MockFontInfo {
            name_string: "Test Font Dziedzic Regular".to_string(),
        };
        
        // Test with a simple substring pattern
        let criteria = QueryCriteria {
            name_patterns: vec!["dzie".to_string()],
            ..Default::default()
        };
        
        // Create regex patterns manually for testing
        let regex_pattern = format!("(?i).*{}.*", regex::escape(&criteria.name_patterns[0]));
        let regex = Regex::new(&regex_pattern).unwrap();
        
        // The pattern should match
        assert!(regex.is_match(&font_info.name_string));
        
        // Test with a case-insensitive pattern
        let criteria = QueryCriteria {
            name_patterns: vec!["DZIE".to_string()],
            ..Default::default()
        };
        
        // Create regex patterns manually for testing
        let regex_pattern = format!("(?i).*{}.*", regex::escape(&criteria.name_patterns[0]));
        let regex = Regex::new(&regex_pattern).unwrap();
        
        // The pattern should match case-insensitively
        assert!(regex.is_match(&font_info.name_string));
        
        // Test with a non-matching pattern
        let criteria = QueryCriteria {
            name_patterns: vec!["nonexistent".to_string()],
            ..Default::default()
        };
        
        // Create regex patterns manually for testing
        let regex_pattern = format!("(?i).*{}.*", regex::escape(&criteria.name_patterns[0]));
        let regex = Regex::new(&regex_pattern).unwrap();
        
        // The pattern should not match
        assert!(!regex.is_match(&font_info.name_string));
    }
} 