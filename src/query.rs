// this_file: fontgrep/src/query.rs
//
// Query execution and font matching

use crate::{
    font::{is_font_file, FontInfo},
    Result,
};
use rayon::prelude::*;
use regex::Regex;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
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
            codepoints.iter().collect()
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
        self.axes.is_empty()
            && self.codepoints.is_empty()
            && self.features.is_empty()
            && self.scripts.is_empty()
            && self.tables.is_empty()
            && self.name_patterns.is_empty()
            && !self.variable
    }

    /// Get the charset query string if codepoints are specified
    pub fn get_charset_query(&self) -> Option<String> {
        if self.codepoints.is_empty() {
            None
        } else {
            // Create a string from the codepoints directly
            let charset: String = self.codepoints.iter().collect();
            Some(charset)
        }
    }
}

/// Font query for executing searches
pub struct FontQuery {
    /// Criteria for the query
    criteria: QueryCriteria,

    /// Number of parallel jobs to use
    jobs: usize,

    /// Compiled name regexes
    name_regexes: Vec<Regex>,
}

impl FontQuery {
    /// Create a new font query
    pub fn new(criteria: QueryCriteria, jobs: usize) -> Self {
        // Compile name regexes
        let name_regexes = criteria
            .name_patterns
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        // Initialize cache if needed
        Self {
            criteria,
            jobs,
            name_regexes,
        }
    }

    /// Execute the query
    pub fn execute(&self, paths: &[PathBuf]) -> Result<Vec<String>> {
        // Collect all font files from the specified paths
        let font_files = self.collect_font_files(paths)?;

        // Process font files in parallel
        let matching_fonts = Arc::new(Mutex::new(Vec::new()));

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
                    let mut fonts = matching_fonts.lock().unwrap();
                    fonts.push(path.to_string_lossy().to_string());
                }
                Ok(false) => {
                    // Font doesn't match criteria
                }
                Err(e) => {
                    eprintln!("Error processing font {}: {}", path.display(), e);
                }
            }
        });

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
                        }
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
            let all_axes_match = self
                .criteria
                .axes
                .iter()
                .all(|axis| font_info.axes.contains(axis));
            if !all_axes_match {
                return Ok(false);
            }
        }

        // Check features
        if !self.criteria.features.is_empty() {
            let all_features_match = self
                .criteria
                .features
                .iter()
                .all(|feature| font_info.features.contains(feature));
            if !all_features_match {
                return Ok(false);
            }
        }

        // Check scripts
        if !self.criteria.scripts.is_empty() {
            let all_scripts_match = self
                .criteria
                .scripts
                .iter()
                .all(|script| font_info.scripts.contains(script));
            if !all_scripts_match {
                return Ok(false);
            }
        }

        // Check tables
        if !self.criteria.tables.is_empty() {
            let all_tables_match = self
                .criteria
                .tables
                .iter()
                .all(|table| font_info.tables.contains(&table.to_string()));
            if !all_tables_match {
                return Ok(false);
            }
        }

        // Check codepoints
        if !self.criteria.codepoints.is_empty() {
            let charset: HashSet<char> = font_info.charset_string.chars().collect();
            let all_codepoints_match = self
                .criteria
                .codepoints
                .iter()
                .all(|cp| charset.contains(cp));
            if !all_codepoints_match {
                return Ok(false);
            }
        }

        // Check name patterns
        if !self.name_regexes.is_empty() {
            let any_name_matches = self
                .name_regexes
                .iter()
                .any(|pattern| pattern.is_match(&font_info.name_string));
            if !any_name_matches {
                return Ok(false);
            }
        }

        Ok(true)
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
            ..Default::default()
        };
        assert_eq!(with_codepoints.get_charset_query(), Some("ABC".to_string()));
    }
}
