// fontgrep - A tool for finding fonts with specific features
//
// this_file: fontgrep/src/main.rs

mod cache;
mod fontinfo;
mod query;

use cache::FontCache;
use clap::Parser;
use query::FontQuery;
use regex::Regex;
use skrifa::Tag;
use std::{
    io::{BufWriter, Write, stdout},
    time::Instant,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Variation axes to find
    #[arg(short, long)]
    axis: Vec<String>,

    /// Codepoints to find (comma-separated list of hyphen-delimited ranges)
    #[arg(short, long, value_parser = parse_unicode_ranges)]
    unicode: Vec<Vec<u32>>,

    // Text support to find (added to --unicode)
    #[arg(short, long)]
    text: Option<String>,

    /// OpenType features to find
    #[arg(short, long)]
    feature: Vec<String>,

    /// Find variable fonts
    #[arg(short, long)]
    variable: bool,

    /// Find fonts with particular tables
    #[arg(short = 'T', long, value_parser = parse_font_tags)]
    table: Vec<Tag>,

    /// Scripts to find
    #[arg(short, long)]
    script: Vec<String>,

    /// Name table entries to find (as regular expressions)
    #[arg(short, long)]
    name: Vec<String>,

    /// Query from cache only. Optionally specify path to filter cached results.
    #[arg(short, long, num_args = 0..=1, default_missing_value = "")]
    cache: Option<String>,

    /// Update cache with fonts from specified path (or current directory if not specified)
    #[arg(short = 'C', long, num_args = 0..=1, default_missing_value = ".")]
    cache_update: Option<String>,

    /// Directory to search for fonts
    #[arg(default_value = ".")]
    directory: Vec<String>,
}

/// Parse a comma-separated list of Unicode ranges
fn parse_unicode_ranges(arg: &str) -> Result<Vec<u32>, String> {
    let mut result = Vec::new();
    
    for range in arg.split(',') {
        let parts: Vec<&str> = range.split('-').collect();
        
        if parts.len() == 1 {
            // Single codepoint
            let codepoint = u32::from_str_radix(parts[0].trim_start_matches("U+"), 16)
                .map_err(|_| format!("Invalid codepoint: {}", parts[0]))?;
            result.push(codepoint);
        } else if parts.len() == 2 {
            // Range of codepoints
            let start = u32::from_str_radix(parts[0].trim_start_matches("U+"), 16)
                .map_err(|_| format!("Invalid start codepoint: {}", parts[0]))?;
            let end = u32::from_str_radix(parts[1].trim_start_matches("U+"), 16)
                .map_err(|_| format!("Invalid end codepoint: {}", parts[1]))?;
            
            for codepoint in start..=end {
                result.push(codepoint);
            }
        } else {
            return Err(format!("Invalid range format: {}", range));
        }
    }
    
    Ok(result)
}

/// Parse a font table tag
fn parse_font_tags(arg: &str) -> Result<Tag, String> {
    if arg.len() != 4 {
        return Err(format!("Table tag must be exactly 4 characters: {}", arg));
    }
    
    let bytes = arg.as_bytes();
    Ok(Tag::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Start timing
    let start_time = Instant::now();
    
    // Process text argument if provided
    let mut unicode_ranges = args.unicode.clone();
    if let Some(text) = &args.text {
        let text_codepoints: Vec<u32> = text.chars().map(|c| c as u32).collect();
        unicode_ranges.push(text_codepoints);
    }
    
    // Compile name regexes
    let name_regexes: Vec<Regex> = args.name.iter()
        .map(|pattern| Regex::new(pattern))
        .collect::<Result<_, _>>()?;
    
    // Initialize cache
    let cache_path = if args.cache.is_some() {
        // Use the cache path from -c if provided, otherwise use default
        args.cache.as_deref().filter(|&p| !p.is_empty())
    } else if args.cache_update.is_some() {
        // Use the cache path from -C if provided, otherwise use default
        args.cache_update.as_deref().filter(|&p| !p.is_empty())
    } else {
        None
    };
    
    let cache = match FontCache::new(cache_path) {
        Ok(cache) => Some(cache),
        Err(e) => {
            eprintln!("Warning: Failed to initialize cache: {}", e);
            None
        }
    };
    
    // Determine directories to search based on command line options
    let search_directories = if args.cache.is_some() {
        // If -c is used, we're only querying the cache
        // If a path is provided with -c, filter cached results by that path
        if let Some(path) = args.cache.as_deref().filter(|&p| !p.is_empty()) {
            vec![path.to_string()]
        } else {
            // Empty vector means query all cached records
            Vec::new()
        }
    } else if args.cache_update.is_some() {
        // If -C is used, we're only updating the cache
        // Use the provided path or "." if none provided
        vec![args.cache_update.as_deref().unwrap_or(".").to_string()]
    } else {
        // Normal mode: search the directories specified on the command line
        args.directory.clone()
    };
    
    // Create and execute the query
    let query = FontQuery::new(
        args.axis,
        unicode_ranges,
        args.feature,
        args.variable,
        args.table,
        args.script,
        name_regexes,
        cache,
    );
    
    // Execute the query based on the mode
    let matching_fonts = if args.cache_update.is_some() {
        // If -C is used, we're only updating the cache, not querying
        // Just update the cache and return empty results
        query.update_cache(&search_directories)?;
        Vec::new()
    } else {
        // Normal query mode or cache query mode
        query.execute(&search_directories)?
    };
    
    // Print results (only if not in cache update mode)
    if args.cache_update.is_none() {
        let stdout = stdout();
        let mut writer = BufWriter::new(stdout.lock());
        
        for font in &matching_fonts {
            writeln!(writer, "{}", font)?;
        }
        
        // Flush the buffer
        writer.flush()?;
        
        // Print summary
        eprintln!(
            "Found {} matching fonts in {:.2} seconds",
            matching_fonts.len(),
            start_time.elapsed().as_secs_f64()
        );
    }
    // We don't need to print a summary for cache update mode anymore
    // as the progress bar already shows completion
    
    Ok(())
} 