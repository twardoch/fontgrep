// this_file: fontgrep/src/cli.rs
//
// Command-line interface for fontgrep

use crate::{
    Result, FontgrepError,
    font::FontInfo,
    query::{FontQuery, QueryCriteria},
};
use clap::{Parser, Subcommand, Args as ClapArgs, ValueEnum};
use skrifa::Tag;
use std::{
    path::PathBuf,
    str::FromStr,
};

/// Command-line arguments for fontgrep
#[derive(Parser, Debug)]
#[command(
    author = "Adam Twardoch <adam@twardoch.com>",
    version,
    about = "A tool to search for fonts based on various criteria",
    long_about = "fontgrep is a command-line tool that helps you find fonts based on their properties, such as OpenType features, variation axes, scripts, and more. It can search through directories of font files and maintain a cache for faster subsequent searches."
)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
    
    /// Path to the cache file
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path to the cache file. Use ':memory:' for an in-memory cache",
        long_help = "Path to the SQLite database file used for caching font information. \
                    If not specified, defaults to a file in the user's data directory. \
                    Use ':memory:' to use an in-memory cache that will be discarded when the program exits."
    )]
    pub cache: Option<String>,
    
    /// Disable the cache
    #[arg(
        long,
        help = "Disable caching and search files directly",
        long_help = "Disable the font cache and search through files directly. \
                    This is slower but ensures you get the most up-to-date results."
    )]
    pub no_cache: bool,
    
    /// Enable verbose output
    #[arg(
        short,
        long,
        help = "Enable verbose output",
        long_help = "Enable verbose output mode that shows additional information \
                    about the search process and font properties."
    )]
    pub verbose: bool,
    
    /// Output format
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = OutputFormat::Text,
        help = "Output format (text, json, or csv)",
        long_help = "Specify the output format:\n\
                    - text: Human-readable text output\n\
                    - json: JSON format for machine processing\n\
                    - csv: CSV format for spreadsheet import"
    )]
    pub format: OutputFormat,
}

/// Subcommands for fontgrep
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search for fonts based on various criteria
    Search(SearchArgs),
    
    /// Update the font cache
    Update(UpdateArgs),
    
    /// Show information about a font
    Info(InfoArgs),
    
    /// List all fonts in the cache
    List,
    
    /// Clean the cache by removing missing fonts
    Clean,
}

/// Arguments for the search command
#[derive(ClapArgs, Debug)]
pub struct SearchArgs {
    /// Directories or font files to search
    #[arg(
        required = true,
        help = "Directories or font files to search",
        long_help = "One or more directories or font files to search. \
                    Directories will be searched recursively for font files."
    )]
    pub paths: Vec<PathBuf>,
    
    /// Variation axes to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Variation axes to search for (e.g., wght,wdth)",
        long_help = "Comma-separated list of OpenType variation axes to search for. \
                    Common axes include:\n\
                    - wght: Weight\n\
                    - wdth: Width\n\
                    - ital: Italic\n\
                    - slnt: Slant\n\
                    - opsz: Optical Size"
    )]
    pub axes: Vec<String>,
    
    /// OpenType features to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "OpenType features to search for (e.g., smcp,onum)",
        long_help = "Comma-separated list of OpenType features to search for. \
                    Common features include:\n\
                    - smcp: Small Capitals\n\
                    - onum: Oldstyle Numerals\n\
                    - liga: Standard Ligatures\n\
                    - kern: Kerning\n\
                    - dlig: Discretionary Ligatures"
    )]
    pub features: Vec<String>,
    
    /// OpenType scripts to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "OpenType scripts to search for (e.g., latn,cyrl)",
        long_help = "Comma-separated list of OpenType script tags to search for. \
                    Common scripts include:\n\
                    - latn: Latin\n\
                    - cyrl: Cyrillic\n\
                    - grek: Greek\n\
                    - arab: Arabic\n\
                    - deva: Devanagari"
    )]
    pub scripts: Vec<String>,
    
    /// Font tables to search for
    #[arg(
        short,
        long,
        value_delimiter = ',',
        help = "Font tables to search for (e.g., GPOS,GSUB)",
        long_help = "Comma-separated list of OpenType table tags to search for. \
                    Common tables include:\n\
                    - GPOS: Glyph Positioning\n\
                    - GSUB: Glyph Substitution\n\
                    - GDEF: Glyph Definition\n\
                    - BASE: Baseline\n\
                    - OS/2: OS/2 and Windows Metrics"
    )]
    pub tables: Vec<String>,
    
    /// Only show variable fonts
    #[arg(
        short,
        long,
        help = "Only show variable fonts",
        long_help = "Only show variable fonts that support OpenType Font Variations."
    )]
    pub variable: bool,
    
    /// Regular expressions to match against font names
    #[arg(
        short,
        long,
        help = "Regular expressions to match against font names",
        long_help = "One or more regular expressions to match against font names. \
                    The search is case-insensitive and matches anywhere in the name."
    )]
    pub name: Vec<String>,
    
    /// Unicode codepoints or ranges to search for
    #[arg(
        short = 'u',
        long,
        value_delimiter = ',',
        help = "Unicode codepoints or ranges to search for (e.g., U+0041-U+005A,U+0061)",
        long_help = "Comma-separated list of Unicode codepoints or ranges to search for. \
                    Formats accepted:\n\
                    - Single codepoint: U+0041 or 0041\n\
                    - Range: U+0041-U+005A\n\
                    - Single character: A"
    )]
    pub codepoints: Vec<String>,
    
    /// Text to check for support
    #[arg(
        short,
        long,
        help = "Text to check for support",
        long_help = "Text string to check for font support. \
                    All characters in the text must be supported by the font."
    )]
    pub text: Option<String>,
    
    /// Number of parallel jobs to use
    #[arg(
        short,
        long,
        default_value_t = num_cpus::get(),
        help = "Number of parallel jobs to use",
        long_help = "Number of parallel jobs to use for searching and processing fonts. \
                    Defaults to the number of CPU cores available."
    )]
    pub jobs: usize,
}

/// Arguments for the update command
#[derive(ClapArgs, Debug)]
pub struct UpdateArgs {
    /// Directories or font files to update in the cache
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,
    
    /// Force update even if the font hasn't changed
    #[arg(short, long)]
    pub force: bool,
    
    /// Number of parallel jobs to use
    #[arg(short, long, default_value_t = num_cpus::get())]
    pub jobs: usize,
}

/// Arguments for the info command
#[derive(ClapArgs, Debug)]
pub struct InfoArgs {
    /// Font file to show information about
    #[arg(required = true)]
    pub path: PathBuf,
    
    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,
}

/// Output format for fontgrep
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output
    Json,
    /// CSV output
    Csv,
}

impl FromStr for OutputFormat {
    type Err = FontgrepError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            _ => Err(FontgrepError::Parse(format!("Invalid output format: {}", s))),
        }
    }
}

/// Parse a list of codepoints from strings
pub fn parse_codepoints(input: &[String]) -> Result<Vec<char>> {
    let mut result = Vec::new();
    
    for item in input {
        if item.contains('-') {
            // Parse a range of codepoints
            let parts: Vec<&str> = item.split('-').collect();
            if parts.len() != 2 {
                return Err(FontgrepError::Parse(format!("Invalid codepoint range: {}", item)));
            }
            
            let start = parse_codepoint(parts[0])?;
            let end = parse_codepoint(parts[1])?;
            
            let start_u32 = start as u32;
            let end_u32 = end as u32;
            
            if start_u32 > end_u32 {
                return Err(FontgrepError::Parse(format!("Invalid codepoint range: {}", item)));
            }
            
            for cp in start_u32..=end_u32 {
                if let Some(c) = char::from_u32(cp) {
                    result.push(c);
                }
            }
        } else {
            // Parse a single codepoint
            let cp = parse_codepoint(item)?;
            result.push(cp);
        }
    }
    
    Ok(result)
}

/// Parse a single codepoint from a string
fn parse_codepoint(input: &str) -> Result<char> {
    let input = input.trim();
    
    // Handle U+XXXX format
    if input.starts_with("U+") || input.starts_with("u+") {
        let hex_str = &input[2..];
        let cp = u32::from_str_radix(hex_str, 16)
            .map_err(|_| FontgrepError::Parse(format!("Invalid codepoint: {}", input)))?;
        
        char::from_u32(cp)
            .ok_or_else(|| FontgrepError::Parse(format!("Invalid codepoint: {}", input)))
    } else if input.len() == 1 {
        // Handle single character
        Ok(input.chars().next().unwrap())
    } else {
        // Try to parse as hex
        let cp = u32::from_str_radix(input, 16)
            .map_err(|_| FontgrepError::Parse(format!("Invalid codepoint: {}", input)))?;
        
        char::from_u32(cp)
            .ok_or_else(|| FontgrepError::Parse(format!("Invalid codepoint: {}", input)))
    }
}

/// Parse a list of table tags from strings
pub fn parse_table_tags(input: &[String]) -> Result<Vec<Tag>> {
    let mut result = Vec::new();
    
    for item in input {
        if item.len() != 4 {
            return Err(FontgrepError::Parse(format!("Invalid table tag: {}", item)));
        }
        
        // Create a Tag from a 4-byte string
        let bytes = item.as_bytes();
        let tag = Tag::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        result.push(tag);
    }
    
    Ok(result)
}

/// Convert CLI arguments to a query criteria
pub fn args_to_query_criteria(args: &SearchArgs) -> Result<QueryCriteria> {
    // Parse codepoints
    let mut codepoints = Vec::new();
    if !args.codepoints.is_empty() {
        codepoints = parse_codepoints(&args.codepoints)?;
    }
    
    // Parse text
    if let Some(text) = &args.text {
        codepoints.extend(text.chars());
    }
    
    // Parse table tags and convert to strings
    let tables_tags = parse_table_tags(&args.tables)?;
    let tables: Vec<String> = tables_tags.iter()
        .map(|tag| tag.to_string())
        .collect();
    
    // Compile name regexes
    let mut name_patterns = Vec::new();
    for pattern in &args.name {
        // Store the pattern string instead of the compiled regex
        name_patterns.push(pattern.clone());
    }
    
    Ok(QueryCriteria::new(
        args.axes.clone(),
        codepoints,
        args.features.clone(),
        args.scripts.clone(),
        tables,
        name_patterns,
        args.variable,
    ))
}

/// Execute the command
pub fn execute(cli: Cli) -> Result<()> {
    // Determine cache path and whether to use cache
    let cache_path = if cli.no_cache {
        None
    } else {
        Some(cli.cache.as_deref())
    };
    
    match &cli.command {
        Commands::Search(args) => {
            // Create query criteria
            let criteria = args_to_query_criteria(args)?;
            
            // Create font query
            let query = FontQuery::new(
                criteria,
                !cli.no_cache,
                cache_path.unwrap_or(None),
                args.jobs,
            );
            
            // Execute query
            let results = query.execute(&args.paths)?;
            
            // Output results
            output_results(&results, cli.format)?;
        }
        Commands::Update(args) => {
            // Create an empty query
            let query = FontQuery::new(
                QueryCriteria::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    false,
                ),
                !cli.no_cache,
                cache_path.unwrap_or(None),
                args.jobs,
            );
            
            // Update cache
            query.update_cache(&args.paths, args.force)?;
            
            println!("Cache updated successfully");
        }
        Commands::Info(args) => {
            // Load font
            let font_info = FontInfo::load(&args.path)?;
            
            // Output font info
            output_font_info(&font_info, args.detailed, cli.format)?;
        }
        Commands::List => {
            // Create an empty query
            let query = FontQuery::new(
                QueryCriteria::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    false,
                ),
                !cli.no_cache,
                cache_path.unwrap_or(None),
                num_cpus::get(),
            );
            
            // List all fonts
            let results = query.list_all_fonts()?;
            
            // Output results
            output_results(&results, cli.format)?;
        }
        Commands::Clean => {
            // Create an empty query
            let query = FontQuery::new(
                QueryCriteria::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    false,
                ),
                !cli.no_cache,
                cache_path.unwrap_or(None),
                num_cpus::get(),
            );
            
            // Clean cache
            query.clean_cache()?;
            
            println!("Cache cleaned successfully");
        }
    }
    
    Ok(())
}

/// Output results in the specified format
fn output_results(results: &[String], format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            for result in results {
                println!("{}", result);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(results)?;
            println!("{}", json);
        }
        OutputFormat::Csv => {
            for result in results {
                println!("{}", result);
            }
        }
    }
    
    Ok(())
}

/// Output font info in the specified format
fn output_font_info(info: &FontInfo, detailed: bool, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!("Name: {}", info.name_string);
            println!("Variable: {}", info.is_variable);
            
            if detailed {
                println!("Axes: {}", info.axes.join(", "));
                println!("Features: {}", info.features.join(", "));
                println!("Scripts: {}", info.scripts.join(", "));
                println!("Tables: {}", info.tables.join(", "));
                println!("Charset: {}", info.charset_string);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(info)?;
            println!("{}", json);
        }
        OutputFormat::Csv => {
            println!("Name,Variable");
            println!("{},{}", info.name_string, info.is_variable);
            
            if detailed {
                println!("Axes,{}", info.axes.join(", "));
                println!("Features,{}", info.features.join(", "));
                println!("Scripts,{}", info.scripts.join(", "));
                println!("Tables,{}", info.tables.join(", "));
                println!("Charset,{}", info.charset_string);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_codepoint() {
        assert_eq!(parse_codepoint("U+0041").unwrap(), 'A');
        assert_eq!(parse_codepoint("u+0041").unwrap(), 'A');
        assert_eq!(parse_codepoint("0041").unwrap(), 'A');
        assert_eq!(parse_codepoint("A").unwrap(), 'A');
        
        assert!(parse_codepoint("U+FFFFFFFF").is_err());
        assert!(parse_codepoint("U+ZZZZ").is_err());
    }
    
    #[test]
    fn test_parse_codepoints() {
        let input = vec!["U+0041".to_string(), "U+0042-U+0044".to_string()];
        let result = parse_codepoints(&input).unwrap();
        assert_eq!(result, vec!['A', 'B', 'C', 'D']);
    }
    
    #[test]
    fn test_parse_table_tags() {
        let input = vec!["GPOS".to_string(), "GSUB".to_string()];
        let result = parse_table_tags(&input).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].to_string(), "GPOS");
        assert_eq!(result[1].to_string(), "GSUB");
    }
} 