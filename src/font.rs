// this_file: fontgrep/src/font.rs
//
// Font information extraction and matching

use crate::{FontgrepError, Result};
use memmap2::Mmap;
use skrifa::prelude::*;
use skrifa::raw::TableProvider;
use skrifa::FontRef;
use std::{
    collections::{BTreeSet, HashSet},
    fs::File,
    path::Path,
};

/// Font information extracted from a font file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FontInfo {
    /// Font name string
    pub(crate) name_string: String,

    /// Whether the font is variable
    pub(crate) is_variable: bool,

    /// Variation axes
    pub(crate) axes: Vec<String>,

    /// OpenType features
    pub(crate) features: Vec<String>,

    /// OpenType scripts
    pub(crate) scripts: Vec<String>,

    /// Font tables
    pub(crate) tables: Vec<String>,

    /// Charset string
    pub(crate) charset_string: String,
}

impl FontInfo {
    /// Load font information from a file
    pub fn load(path: &Path) -> Result<Self> {
        let font = load_font(path)?;
        Self::from_font(&font)
    }

    /// Extract font information from a font reference
    fn from_font(font: &FontRef) -> Result<Self> {
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
}

/// Load a font from a file with optimized memory mapping
fn load_font(path: &Path) -> Result<FontRef<'static>> {
    let file = File::open(path)?;
    let data = Box::leak(Box::new(unsafe {
        Mmap::map(&file).map_err(|e| FontgrepError::Mmap(e.to_string()))?
    }));
    FontRef::new(data).map_err(|e| FontgrepError::Font(e.to_string()))
}

/// Check if a file is a font based on its extension
pub(crate) fn is_font_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "ttf" | "otf" | "ttc" | "otc")
    } else {
        false
    }
}
/// Create a charset from a font with optimized implementation
fn create_charset(font: &FontRef) -> BTreeSet<u32> {
    let mut charset = BTreeSet::new();

    // Get the character map from the font
    let charmap = font.charmap();

    // If the font has a character map, extract all supported codepoints
    if charmap.has_map() {
        // Use the mappings() method to get all codepoint to glyph mappings
        for (codepoint, _glyph_id) in charmap.mappings() {
            // Skip invalid Unicode codepoints
            charset.insert(codepoint);
        }
    }

    charset
}

/// Convert a charset to a string with optimized implementation
fn charset_to_string(charset: &BTreeSet<u32>) -> String {
    let mut result = String::with_capacity(charset.len());
    for &cp in charset {
        if let Some(c) = char::from_u32(cp) {
            result.push(c);
        }
    }
    result
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

#[cfg(test)]
mod tests {
    use super::*;

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
