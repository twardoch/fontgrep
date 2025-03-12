// this_file: fontgrep/src/query.rs
//
// Query execution and font matching

use crate::{
    cli::SearchArgs,
    font::{is_font_file, FontInfo},
    matchers::{
        AxesMatcher, CodepointsMatcher, FeaturesMatcher, FontMatcher, NameMatcher, ScriptsMatcher,
        TablesMatcher,
    },
    Result,
};
use rayon::prelude::*;
use skrifa::Tag;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

/// Criteria for querying fonts
#[derive(Default)]
pub struct FontQuery {
    matchers: Vec<Box<dyn FontMatcher>>,
    jobs: usize,
    paths: Vec<PathBuf>,
}

// It's fine.
unsafe impl Sync for FontQuery {}

impl From<&SearchArgs> for FontQuery {
    fn from(args: &SearchArgs) -> Self {
        let mut matchers: Vec<Box<dyn FontMatcher>> = Vec::new();

        // Matches should be added from quickest / most effective filter to slowest
        if args.variable {
            matchers.push(Box::new(TablesMatcher::new(&[Tag::new(b"fvar")])));
        }

        if !args.tables.is_empty() {
            matchers.push(Box::new(TablesMatcher::new(&args.tables)));
        }

        if !args.axes.is_empty() {
            matchers.push(Box::new(AxesMatcher::new(&args.axes)));
        }

        if !args.features.is_empty() {
            matchers.push(Box::new(FeaturesMatcher::new(&args.features)));
        }

        if !args.scripts.is_empty() {
            matchers.push(Box::new(ScriptsMatcher::new(&args.scripts)));
        }

        if !args.name.is_empty() {
            matchers.push(Box::new(NameMatcher::new(&args.name)));
        }

        if !args.codepoints.is_empty() || args.text.is_some() {
            let mut codepoints: Vec<char> = Vec::new();
            if let Some(text) = &args.text {
                codepoints.extend(text.chars());
            }
            codepoints.extend(&args.codepoints);
            matchers.push(Box::new(CodepointsMatcher::new(&codepoints)));
        }

        Self {
            matchers,
            jobs: args.jobs,
            paths: args.paths.clone(),
        }
    }
}

impl FontQuery {
    /// Execute the query
    pub fn execute(&self) -> Result<Vec<String>> {
        // Collect all font files from the specified paths
        let font_files = self.collect_font_files(&self.paths)?;

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

        Ok(self
            .matchers
            .iter()
            .all(|matcher| matcher.matches(&font_info)))
    }
}
