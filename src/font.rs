// this_file: fontgrep/src/font.rs
//
// Font information extraction and matching

use crate::{FontgrepError, Result};
use memmap2::Mmap;
use skrifa::prelude::*;
use skrifa::raw::TableProvider;
use skrifa::{FontRef, Tag};
use std::{
    collections::{BTreeSet, HashSet},
    fs::File,
    path::Path,
};

/// Font information extracted from a font file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FontInfo {
    /// Font name string
    pub name_string: String,

    /// Whether the font is variable
    pub is_variable: bool,

    /// Variation axes
    pub axes: Vec<String>,

    /// OpenType features
    pub features: Vec<String>,

    /// OpenType scripts
    pub scripts: Vec<String>,

    /// Font tables
    pub tables: Vec<String>,

    /// Charset string
    pub charset_string: String,
}

impl FontInfo {
    /// Load font information from a file
    pub fn load(path: &Path) -> Result<Self> {
        let font = load_font(path)?;
        Self::from_font(&font)
    }

    /// Extract font information from a font reference
    pub fn from_font(font: &FontRef) -> Result<Self> {
        // Extract name string with error handling
        let name_string = extract_name_string(font);

        // Check if font is variable with error handling
        let is_variable = has_variations(font);

        // Extract variation axes with error handling
        let axes = extract_axes(font);

        // Extract OpenType features with error handling
        let features = extract_features(font);

        // Extract OpenType scripts with error handling
        let scripts = extract_scripts(font);

        // Extract font tables with error handling
        let tables = extract_tables(font);

        // Create charset with optimized implementation
        let charset = create_charset(font);
        let charset_string = charset_to_string(&charset);

        Ok(FontInfo {
            name_string,
            is_variable,
            axes,
            features,
            scripts,
            tables,
            charset_string,
        })
    }

    /// Check if a file is a font file based on its extension
    pub fn is_font_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            matches!(ext_str.as_str(), "ttf" | "otf" | "ttc" | "otc")
        } else {
            false
        }
    }
}

/// Load a font from a file with optimized memory mapping
pub fn load_font(path: &Path) -> Result<FontRef<'static>> {
    let file = File::open(path)?;
    let data = Box::leak(Box::new(unsafe {
        Mmap::map(&file).map_err(|e| FontgrepError::Io(e.to_string()))?
    }));
    FontRef::new(data).map_err(|e| FontgrepError::Font(e.to_string()))
}

/// Check if a file is a font based on its extension
pub fn is_font_file(path: &Path) -> bool {
    FontInfo::is_font_file(path)
}

/// Create a charset from a font with optimized implementation
pub fn create_charset(font: &FontRef) -> BTreeSet<u32> {
    let mut charset = BTreeSet::new();

    // Get the character map from the font
    let charmap = font.charmap();

    // If the font has a character map, extract all supported codepoints
    if charmap.has_map() {
        // Use the mappings() method to get all codepoint to glyph mappings
        for (codepoint, _glyph_id) in charmap.mappings() {
            // Skip invalid Unicode codepoints
            if !is_invalid_unicode(codepoint) {
                charset.insert(codepoint);
            }
        }
    }

    charset
}

/// Convert a charset to a string with optimized implementation
pub fn charset_to_string(charset: &BTreeSet<u32>) -> String {
    let mut result = String::with_capacity(charset.len());
    for &cp in charset {
        if let Some(c) = char::from_u32(cp) {
            result.push(c);
        }
    }
    result
}

/// Check if a Unicode codepoint is invalid or problematic
fn is_invalid_unicode(codepoint: u32) -> bool {
    // U+0000 (NULL)
    // U+0001-U+001F (C0 controls)
    // U+007F (DELETE)
    // U+0080-U+009F (C1 controls)
    // U+D800-U+DFFF (surrogate pairs)
    // U+FDD0-U+FDEF (noncharacters)
    // U+FFFE, U+FFFF (noncharacters)
    // U+1FFFE, U+1FFFF, U+2FFFE, U+2FFFF, ... U+10FFFE, U+10FFFF (noncharacters)

    (codepoint <= 0x001F)
        || (codepoint == 0x007F)
        || (codepoint >= 0x0080 && codepoint <= 0x009F)
        || (codepoint >= 0xD800 && codepoint <= 0xDFFF)
        || (codepoint >= 0xFDD0 && codepoint <= 0xFDEF)
        || (codepoint == 0xFFFE || codepoint == 0xFFFF)
        || (codepoint & 0xFFFE) == 0xFFFE && codepoint <= 0x10FFFF
}

/// Extract the name string from a font with improved name record handling
fn extract_name_string(font: &FontRef) -> String {
    let mut name_strings = HashSet::new();

    if let Ok(name) = font.name() {
        // Extract all name records
        for record in name.name_record() {
            if let Ok(string) = record.string(name.string_data()) {
                name_strings.insert(string.to_string());
            }
        }
    }

    name_strings.into_iter().collect::<Vec<_>>().join(" ")
}

/// Check if a font has variations
fn has_variations(font: &FontRef) -> bool {
    !font.axes().is_empty()
}

/// Extract variation axes from a font
fn extract_axes(font: &FontRef) -> Vec<String> {
    font.axes()
        .iter()
        .map(|axis| axis.tag().to_string())
        .collect()
}

/// Extract OpenType features from a font
fn extract_features(font: &FontRef) -> Vec<String> {
    let mut features = HashSet::new();

    // Extract GSUB features
    if let Ok(gsub) = font.gsub() {
        if let Ok(feature_list) = gsub.feature_list() {
            for feature in feature_list.feature_records() {
                features.insert(feature.feature_tag().to_string());
            }
        }
    }

    // Extract GPOS features
    if let Ok(gpos) = font.gpos() {
        if let Ok(feature_list) = gpos.feature_list() {
            for feature in feature_list.feature_records() {
                features.insert(feature.feature_tag().to_string());
            }
        }
    }

    features.into_iter().collect()
}

/// Extract OpenType scripts from a font
fn extract_scripts(font: &FontRef) -> Vec<String> {
    let mut scripts = HashSet::new();

    // Extract GSUB scripts
    if let Ok(gsub) = font.gsub() {
        if let Ok(script_list) = gsub.script_list() {
            for script in script_list.script_records() {
                scripts.insert(script.script_tag().to_string());
            }
        }
    }

    // Extract GPOS scripts
    if let Ok(gpos) = font.gpos() {
        if let Ok(script_list) = gpos.script_list() {
            for script in script_list.script_records() {
                scripts.insert(script.script_tag().to_string());
            }
        }
    }

    scripts.into_iter().collect()
}

/// Extract font tables from a font
fn extract_tables(font: &FontRef) -> Vec<String> {
    font.table_directory
        .table_records()
        .iter()
        .map(|record| record.tag().to_string())
        .collect()
}

/// Trait for matching fonts
pub trait FontMatcher {
    /// Check if a font matches the criteria
    fn matches(&self, info: &FontInfo) -> bool;
}

/// Matcher for variation axes
pub struct AxesMatcher {
    axes: Vec<String>,
}

impl AxesMatcher {
    /// Create a new axes matcher
    pub fn new(axes: &[String]) -> Self {
        Self {
            axes: axes.to_vec(),
        }
    }
}

impl FontMatcher for AxesMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        self.axes.iter().all(|axis| info.axes.contains(axis))
    }
}

/// Matcher for OpenType features
pub struct FeaturesMatcher {
    features: Vec<String>,
}

impl FeaturesMatcher {
    /// Create a new features matcher
    pub fn new(features: &[String]) -> Self {
        Self {
            features: features.to_vec(),
        }
    }
}

impl FontMatcher for FeaturesMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        self.features
            .iter()
            .all(|feature| info.features.contains(feature))
    }
}

/// Matcher for OpenType scripts
pub struct ScriptsMatcher {
    scripts: Vec<String>,
}

impl ScriptsMatcher {
    /// Create a new scripts matcher
    pub fn new(scripts: &[String]) -> Self {
        Self {
            scripts: scripts.to_vec(),
        }
    }
}

impl FontMatcher for ScriptsMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        self.scripts
            .iter()
            .all(|script| info.scripts.contains(script))
    }
}

/// Matcher for font tables
pub struct TablesMatcher {
    tables: Vec<Tag>,
}

impl TablesMatcher {
    /// Create a new tables matcher
    pub fn new(tables: &[Tag]) -> Self {
        Self {
            tables: tables.to_vec(),
        }
    }
}

impl FontMatcher for TablesMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        self.tables
            .iter()
            .all(|table| info.tables.contains(&table.to_string()))
    }
}

/// Matcher for variable fonts
pub struct VariableFontMatcher;

impl VariableFontMatcher {
    /// Create a new variable font matcher
    pub fn new() -> Self {
        Self
    }
}

impl FontMatcher for VariableFontMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        info.is_variable
    }
}

/// Matcher for Unicode codepoints
pub struct CodepointsMatcher {
    codepoints: Vec<char>,
}

impl CodepointsMatcher {
    /// Create a new codepoints matcher
    pub fn new(codepoints: &[char]) -> Self {
        Self {
            codepoints: codepoints.to_vec(),
        }
    }
}

impl FontMatcher for CodepointsMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        let charset: HashSet<char> = info.charset_string.chars().collect();
        self.codepoints.iter().all(|cp| charset.contains(cp))
    }
}

/// Matcher for font names
pub struct NameMatcher {
    patterns: Vec<regex::Regex>,
}

impl NameMatcher {
    /// Create a new name matcher
    pub fn new(patterns: &[regex::Regex]) -> Self {
        Self {
            patterns: patterns.to_vec(),
        }
    }
}

impl FontMatcher for NameMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        self.patterns
            .iter()
            .any(|pattern| pattern.is_match(&info.name_string))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_invalid_unicode() {
        // Control characters
        assert!(is_invalid_unicode(0x0000));
        assert!(is_invalid_unicode(0x001F));
        assert!(is_invalid_unicode(0x007F));
        assert!(is_invalid_unicode(0x0080));
        assert!(is_invalid_unicode(0x009F));

        // Surrogate pairs
        assert!(is_invalid_unicode(0xD800));
        assert!(is_invalid_unicode(0xDFFF));

        // Noncharacters
        assert!(is_invalid_unicode(0xFDD0));
        assert!(is_invalid_unicode(0xFDEF));
        assert!(is_invalid_unicode(0xFFFE));
        assert!(is_invalid_unicode(0xFFFF));
        assert!(is_invalid_unicode(0x1FFFE));
        assert!(is_invalid_unicode(0x10FFFE));

        // Valid codepoints
        assert!(!is_invalid_unicode(0x0041)); // 'A'
        assert!(!is_invalid_unicode(0x1F600)); // 😀
    }

    #[test]
    fn test_is_font_file() {
        assert!(is_font_file(Path::new("test.ttf")));
        assert!(is_font_file(Path::new("test.otf")));
        assert!(is_font_file(Path::new("test.ttc")));
        assert!(is_font_file(Path::new("test.otc")));
        assert!(is_font_file(Path::new("test.TTF")));

        assert!(!is_font_file(Path::new("test.txt")));
        assert!(!is_font_file(Path::new("test")));
    }
}
