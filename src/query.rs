// this_file: fontgrep/src/query.rs

use crate::cache::{FontCache, get_file_mtime, get_file_size};
use crate::fontinfo::FontInfo;
use jwalk::WalkDir;
use memmap2::Mmap;
use read_fonts::TableProvider;
use regex::Regex;
use skrifa::{FontRef, MetadataProvider, Tag};
use std::{
    collections::HashSet,
    fs::File,
    io,
    path::Path,
    sync::{Arc, Mutex},
};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress, ParallelProgressIterator};

/// Represents a query for fonts with specific criteria
pub struct FontQuery {
    /// Variation axes to find
    pub axes: Vec<String>,
    
    /// Unicode codepoints to find
    pub codepoints: Vec<Vec<u32>>,
    
    /// OpenType features to find
    pub features: Vec<String>,
    
    /// Whether to find only variable fonts
    pub variable: bool,
    
    /// OpenType tables to find
    pub tables: Vec<Tag>,
    
    /// Scripts to find
    pub scripts: Vec<String>,
    
    /// Name table entries to find (as regular expressions)
    pub name_regexes: Vec<Regex>,
    
    /// Cache to use for the query
    pub cache: Option<Arc<Mutex<FontCache>>>,
}

impl FontQuery {
    /// Creates a new font query
    pub fn new(
        axes: Vec<String>,
        codepoints: Vec<Vec<u32>>,
        features: Vec<String>,
        variable: bool,
        tables: Vec<Tag>,
        scripts: Vec<String>,
        name_regexes: Vec<Regex>,
        cache: Option<FontCache>,
    ) -> Self {
        FontQuery {
            axes,
            codepoints,
            features,
            variable,
            tables,
            scripts,
            name_regexes,
            cache: cache.map(|c| Arc::new(Mutex::new(c))),
        }
    }
    
    /// Executes the query on a directory and returns matching font paths
    pub fn execute(&self, directories: &[String]) -> io::Result<Vec<String>> {
        let matching_fonts = Arc::new(Mutex::new(Vec::new()));
        let existing_paths = Arc::new(Mutex::new(HashSet::new()));
        
        // Check if we're querying only from cache
        if let Some(cache_arc) = &self.cache {
            if directories.is_empty() {
                // Empty directories means query all cached records
                return self.query_cache_all(cache_arc.clone());
            } else if directories.len() == 1 && !directories[0].is_empty() {
                // Single directory with path means filter cached records by path
                return self.query_cache_filtered(cache_arc.clone(), &directories[0]);
            }
        }
        
        // Process each directory in parallel
        directories.par_iter().for_each(|dir| {
            let result = self.search_directory(
                dir,
                matching_fonts.clone(),
                existing_paths.clone(),
            );
            
            if let Err(e) = result {
                eprintln!("Error searching directory {}: {}", dir, e);
            }
        });
        
        // Clean up the cache if we have one
        if let Some(cache_arc) = &self.cache {
            let mut cache = cache_arc.lock().unwrap();
            if let Err(e) = cache.clean_missing_fonts(&existing_paths.lock().unwrap()) {
                eprintln!("Error cleaning cache: {}", e);
            }
        }
        
        // Return the matching fonts
        let result = matching_fonts.lock().unwrap().clone();
        Ok(result)
    }
    
    /// Queries all fonts from the cache
    fn query_cache_all(&self, cache_arc: Arc<Mutex<FontCache>>) -> io::Result<Vec<String>> {
        let cache = cache_arc.lock().unwrap();
        let charset_query = self.get_charset_query();
        
        match cache.query(
            &self.axes,
            &self.features,
            &self.scripts,
            &self.tables,
            &self.name_regexes.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
            self.variable,
            charset_query.as_deref(),
        ) {
            Ok(paths) => Ok(paths),
            Err(e) => {
                eprintln!("Error querying cache: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    /// Queries fonts from the cache filtered by path
    fn query_cache_filtered(&self, cache_arc: Arc<Mutex<FontCache>>, path_filter: &str) -> io::Result<Vec<String>> {
        let cache = cache_arc.lock().unwrap();
        let charset_query = self.get_charset_query();
        
        // Add a path filter to the query
        match cache.query(
            &self.axes,
            &self.features,
            &self.scripts,
            &self.tables,
            &self.name_regexes.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
            self.variable,
            charset_query.as_deref(),
        ) {
            Ok(paths) => {
                // Filter the results by path
                let filtered_paths = paths
                    .into_iter()
                    .filter(|p| p.starts_with(path_filter))
                    .collect();
                Ok(filtered_paths)
            }
            Err(e) => {
                eprintln!("Error querying cache: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    /// Searches a directory for fonts matching the query
    fn search_directory(
        &self,
        directory: &str,
        matching_fonts: Arc<Mutex<Vec<String>>>,
        existing_paths: Arc<Mutex<HashSet<String>>>,
    ) -> io::Result<()> {
        // Create a WalkDir iterator with 8 threads
        let walker = WalkDir::new(directory)
            .skip_hidden(false)
            .process_read_dir(|_, _, _, dir_entry_results| {
                // Filter out entries that are not font files
                dir_entry_results.retain(|entry| {
                    if let Ok(entry) = entry {
                        // Skip directories
                        if entry.file_type().is_dir() {
                            return true;
                        }
                        
                        // Check if it's a font file
                        if let Some(name) = entry.file_name().to_str() {
                            return is_font_file(name);
                        }
                    }
                    false
                });
            });
        
        // Process each entry in parallel
        walker.into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .for_each(|entry| {
                // Skip directories
                if entry.file_type().is_dir() {
                    return;
                }
                
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();
                
                // Add to existing paths
                existing_paths.lock().unwrap().insert(path_str.clone());
                
                // Check if we can use the cache
                if let Some(cache_arc) = &self.cache {
                    // Get file metadata
                    if let (Ok(mtime), Ok(size)) = (get_file_mtime(&path), get_file_size(&path)) {
                        // Check if the font is in the cache and up to date
                        let cache = cache_arc.lock().unwrap();
                        match cache.needs_update(&path_str, mtime, size) {
                            Ok(false) => {
                                // Font is in cache and up to date, check if it matches
                                let charset_query = self.get_charset_query();
                                match cache.font_matches(
                                    &path_str,
                                    &self.axes,
                                    &self.features,
                                    &self.scripts,
                                    &self.tables,
                                    &self.name_regexes.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
                                    self.variable,
                                    charset_query.as_deref(),
                                ) {
                                    Ok(true) => {
                                        // Font matches, add to results
                                        matching_fonts.lock().unwrap().push(path_str);
                                    }
                                    _ => {
                                        // Font doesn't match or error occurred
                                    }
                                }
                                return;
                            }
                            _ => {
                                // Font is not in cache or needs updating, continue with normal processing
                                drop(cache); // Explicitly drop the lock
                            }
                        }
                    }
                }
                
                // Process the font file directly
                if let Ok(()) = self.process_font_file(&path, matching_fonts.clone()) {
                    // Successfully processed
                }
            });
        
        Ok(())
    }
    
    /// Processes a single font file
    fn process_font_file(&self, path: &Path, matching_fonts: Arc<Mutex<Vec<String>>>) -> io::Result<()> {
        // Open the file
        let file = File::open(path)?;
        let data = unsafe { Mmap::map(&file)? };
        
        // Parse the font
        let font = match FontRef::new(&data) {
            Ok(font) => font,
            Err(_) => return Ok(()), // Skip invalid fonts
        };
        
        // Check if the font matches the query
        if self.font_matches(&font) {
            // Add to matching fonts
            let path_str = path.to_string_lossy().to_string();
            matching_fonts.lock().unwrap().push(path_str.clone());
            
            // Update the cache if we have one
            if let Some(cache_arc) = &self.cache {
                let font_info = FontInfo::from_font(&font);
                if let (Ok(mtime), Ok(size)) = (get_file_mtime(path), get_file_size(path)) {
                    let mut cache = cache_arc.lock().unwrap();
                    if let Err(e) = cache.update_font(&path_str, &font_info, mtime, size) {
                        eprintln!("Error updating cache for {}: {}", path_str, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Checks if a font matches the query criteria
    fn font_matches(&self, font: &FontRef) -> bool {
        // Variable font check
        if self.variable && font.axes().is_empty() {
            return false;
        }
        
        // Axis filters
        for axis in &self.axes {
            if !axis_filter(font, axis) {
                return false;
            }
        }
        
        // Feature filters
        for feature in &self.features {
            if !feature_filter(font, feature) {
                return false;
            }
        }
        
        // Script filters
        for script in &self.scripts {
            if !script_filter(font, script) {
                return false;
            }
        }
        
        // Table filters
        for table in &self.tables {
            if !table_filter(font, *table) {
                return false;
            }
        }
        
        // Codepoint filters - each range is treated as an OR group, but all ranges must match
        for codepoint_range in &self.codepoints {
            // For each range, at least one codepoint must be supported
            // If the range is empty, consider it a match
            if !codepoint_range.is_empty() {
                // For text queries (which are typically a single range with multiple characters),
                // we want to require ALL characters to be present
                let is_text_query = codepoint_range.len() > 1 && 
                                   codepoint_range.iter().all(|cp| *cp < 0x10000 && std::char::from_u32(*cp).is_some());
                
                let range_match = if is_text_query {
                    // For text queries, ALL codepoints must match
                    codepoint_range.iter().all(|codepoint| codepoint_filter(font, *codepoint))
                } else {
                    // For regular Unicode ranges, ANY codepoint can match
                    codepoint_range.iter().any(|codepoint| codepoint_filter(font, *codepoint))
                };
                
                if !range_match {
                    return false;
                }
            }
        }
        
        // Name regex filters
        for regex in &self.name_regexes {
            if !name_filter(font, regex) {
                return false;
            }
        }
        
        true
    }
    
    /// Gets a string representation of the codepoints for cache queries
    fn get_charset_query(&self) -> Option<String> {
        if self.codepoints.is_empty() {
            return None;
        }
        
        // For text queries (which are typically a single range with multiple characters),
        // we want to require ALL characters to be present
        for range in &self.codepoints {
            if range.len() > 1 && range.iter().all(|cp| *cp < 0x10000 && std::char::from_u32(*cp).is_some()) {
                // This is likely a text query, so return all characters as a single string
                // The cache query will use LIKE %char% for each character, effectively requiring ALL
                return Some(range.iter()
                    .filter_map(|cp| std::char::from_u32(*cp))
                    .collect());
            }
        }
        
        // For regular Unicode ranges, we'll just return the first few codepoints
        // as a sample to query the cache
        let mut chars = String::new();
        let mut count = 0;
        
        for range in &self.codepoints {
            for cp in range {
                if let Some(c) = std::char::from_u32(*cp) {
                    chars.push(c);
                    count += 1;
                    if count >= 5 {
                        break;
                    }
                }
            }
            if count >= 5 {
                break;
            }
        }
        
        if chars.is_empty() {
            None
        } else {
            Some(chars)
        }
    }

    /// Updates the cache with fonts from the specified directories without performing a query
    pub fn update_cache(&self, directories: &[String]) -> io::Result<()> {
        // Check if we have a cache
        let cache_arc = match &self.cache {
            Some(cache) => cache.clone(),
            None => return Ok(()),
        };
        
        let existing_paths = Arc::new(Mutex::new(HashSet::new()));
        let multi_progress = MultiProgress::new();
        
        // Create a spinner for the initial scan
        let spinner = multi_progress.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        spinner.set_message("Scanning directories for font files...");
        
        // First, count the total number of font files to process
        let mut total_files = 0;
        for dir in directories {
            let walker = WalkDir::new(dir)
                .skip_hidden(false)
                .process_read_dir(|_, _, _, dir_entry_results| {
                    // Filter out entries that are not font files
                    dir_entry_results.retain(|entry| {
                        if let Ok(entry) = entry {
                            // Skip directories
                            if entry.file_type().is_dir() {
                                return true;
                            }
                            
                            // Check if it's a font file
                            if let Some(name) = entry.file_name().to_str() {
                                return is_font_file(name);
                            }
                        }
                        false
                    });
                });
            
            for entry in walker.into_iter().filter_map(Result::ok) {
                if !entry.file_type().is_dir() {
                    total_files += 1;
                }
            }
        }
        
        spinner.finish_with_message(format!("Found {} font files to process", total_files));
        
        // Create a progress bar for the cache update
        let progress_bar = multi_progress.add(ProgressBar::new(total_files as u64));
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        progress_bar.set_message("Updating cache...");
        
        // Process each directory in parallel
        let progress_clone = progress_bar.clone();
        directories.par_iter().for_each(|dir| {
            let result = self.update_cache_for_directory(
                dir,
                cache_arc.clone(),
                existing_paths.clone(),
                progress_clone.clone(),
            );
            
            if let Err(e) = result {
                eprintln!("Error updating cache for directory {}: {}", dir, e);
            }
        });
        
        progress_bar.finish_with_message("Cache update completed");
        
        // Clean up the cache
        let mut cache = cache_arc.lock().unwrap();
        if let Err(e) = cache.clean_missing_fonts(&existing_paths.lock().unwrap()) {
            eprintln!("Error cleaning cache: {}", e);
        }
        
        Ok(())
    }
    
    /// Updates the cache with fonts from a single directory
    fn update_cache_for_directory(
        &self,
        directory: &str,
        cache_arc: Arc<Mutex<FontCache>>,
        existing_paths: Arc<Mutex<HashSet<String>>>,
        progress_bar: ProgressBar,
    ) -> io::Result<()> {
        // Create a WalkDir iterator with 8 threads
        let walker = WalkDir::new(directory)
            .skip_hidden(false)
            .process_read_dir(|_, _, _, dir_entry_results| {
                // Filter out entries that are not font files
                dir_entry_results.retain(|entry| {
                    if let Ok(entry) = entry {
                        // Skip directories
                        if entry.file_type().is_dir() {
                            return true;
                        }
                        
                        // Check if it's a font file
                        if let Some(name) = entry.file_name().to_str() {
                            return is_font_file(name);
                        }
                    }
                    false
                });
            });
        
        // Process each entry in parallel
        walker.into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .for_each(|entry| {
                // Skip directories
                if entry.file_type().is_dir() {
                    return;
                }
                
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();
                
                // Add to existing paths
                existing_paths.lock().unwrap().insert(path_str.clone());
                
                // Get file metadata
                if let (Ok(mtime), Ok(size)) = (get_file_mtime(&path), get_file_size(&path)) {
                    // Check if the font is in the cache and up to date
                    let cache = cache_arc.lock().unwrap();
                    match cache.needs_update(&path_str, mtime, size) {
                        Ok(false) => {
                            // Font is in cache and up to date, nothing to do
                            progress_bar.inc(1);
                            return;
                        }
                        _ => {
                            // Font is not in cache or needs updating
                            drop(cache); // Explicitly drop the lock
                        }
                    }
                }
                
                // Process the font file and update the cache
                self.update_cache_for_file(&path, cache_arc.clone());
                progress_bar.inc(1);
            });
        
        Ok(())
    }
    
    /// Updates the cache for a single font file
    fn update_cache_for_file(&self, path: &Path, cache_arc: Arc<Mutex<FontCache>>) {
        // Open the file
        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening file {}: {}", path.display(), e);
                return;
            }
        };
        
        let data = match unsafe { Mmap::map(&file) } {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error mapping file {}: {}", path.display(), e);
                return;
            }
        };
        
        // Parse the font
        let font = match FontRef::new(&data) {
            Ok(font) => font,
            Err(_) => return, // Skip invalid fonts
        };
        
        // Extract font info and update the cache
        let font_info = FontInfo::from_font(&font);
        let path_str = path.to_string_lossy().to_string();
        
        if let (Ok(mtime), Ok(size)) = (get_file_mtime(path), get_file_size(path)) {
            let mut cache = cache_arc.lock().unwrap();
            if let Err(e) = cache.update_font(&path_str, &font_info, mtime, size) {
                eprintln!("Error updating cache for {}: {}", path_str, e);
            }
        }
    }
}

/// Checks if a file is a font file based on its extension
pub fn is_font_file(name: &str) -> bool {
    let name = name.to_lowercase();
    if name.ends_with(".ttf") || name.ends_with(".otf") || 
       name.ends_with(".ttc") || name.ends_with(".otc") ||
       name.ends_with(".woff") || name.ends_with(".woff2") ||
       name.ends_with(".dfont") || name.ends_with(".pfa") || 
       name.ends_with(".pfb") || name.ends_with(".eot") {
        return true;
    }
    false
}

/// Checks if a font has a specific variation axis
fn axis_filter(font: &FontRef, axis: &str) -> bool {
    font.axes().iter().any(|a| a.tag().to_string() == axis)
}

/// Checks if a font has a specific OpenType feature
fn feature_filter(font: &FontRef, feature: &str) -> bool {
    // Check GSUB features
    if let Ok(gsub) = font.gsub() {
        if let Ok(feature_list) = gsub.feature_list() {
            for f in feature_list.feature_records() {
                if f.feature_tag() == feature {
                    return true;
                }
            }
        }
    }
    
    // Check GPOS features
    if let Ok(gpos) = font.gpos() {
        if let Ok(feature_list) = gpos.feature_list() {
            for f in feature_list.feature_records() {
                if f.feature_tag() == feature {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Checks if a font has a specific OpenType table
fn table_filter(font: &FontRef, table: Tag) -> bool {
    font.table_data(table).is_some()
}

/// Checks if a font supports a specific script
fn script_filter(font: &FontRef, script: &str) -> bool {
    // Check GSUB scripts
    if let Ok(gsub) = font.gsub() {
        if let Ok(script_list) = gsub.script_list() {
            for s in script_list.script_records() {
                if s.script_tag() == script {
                    return true;
                }
            }
        }
    }
    
    // Check GPOS scripts
    if let Ok(gpos) = font.gpos() {
        if let Ok(script_list) = gpos.script_list() {
            for s in script_list.script_records() {
                if s.script_tag() == script {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Checks if a font supports a specific Unicode codepoint
fn codepoint_filter(font: &FontRef, codepoint: u32) -> bool {
    font.charmap().map(codepoint).is_some()
}

/// Checks if a font's name table matches a regex
fn name_filter(font: &FontRef, regex: &Regex) -> bool {
    if let Ok(name) = font.name() {
        for record in name.name_record() {
            if let Ok(string) = record.string(name.string_data()) {
                if regex.is_match(&string.to_string()) {
                    return true;
                }
            }
        }
    }
    
    false
} 