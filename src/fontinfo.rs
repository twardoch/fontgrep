// this_file: fontgrep/src/fontinfo.rs

use skrifa::{FontRef, MetadataProvider, Tag};
use read_fonts::{TableProvider, TableDirectory};
use std::collections::HashSet;

/// Represents the extracted information from a font file
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// All OpenType table tags present in the font
    pub tables: Vec<Tag>,
    
    /// All variation axis tags (from fvar table)
    pub axes: Vec<Tag>,
    
    /// All GSUB and GPOS feature tags
    pub features: Vec<String>,
    
    /// All GSUB and GPOS script tags
    pub scripts: Vec<String>,
    
    /// All strings from name table, deduped and space-concatenated
    pub name_string: String,
    
    /// Unicode codepoints supported by the font as a sorted string
    pub charset_string: String,
    
    /// Whether the font is variable
    pub is_variable: bool,
}

impl FontInfo {
    /// Extract all relevant information from a font file
    pub fn from_font(font: &FontRef) -> Self {
        FontInfo {
            tables: extract_tables(font),
            axes: extract_axes(font),
            features: extract_features(font),
            scripts: extract_scripts(font),
            name_string: extract_name_string(font),
            charset_string: create_charset_string(font),
            is_variable: !font.axes().is_empty(),
        }
    }
}

/// Extract all table tags from a font
fn extract_tables(font: &FontRef) -> Vec<Tag> {
    font.table_directory.table_records().iter().map(|record| record.tag()).collect()
}

/// Extract all variation axis tags from a font
fn extract_axes(font: &FontRef) -> Vec<Tag> {
    font.axes().iter().map(|axis| axis.tag()).collect()
}

/// Extract all GSUB and GPOS feature tags from a font
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

/// Extract all GSUB and GPOS script tags from a font
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

/// Extract all strings from the name table, deduplicated and space-concatenated
fn extract_name_string(font: &FontRef) -> String {
    let mut name_strings = HashSet::new();
    
    if let Ok(name) = font.name() {
        for record in name.name_record() {
            if let Ok(string) = record.string(name.string_data()) {
                name_strings.insert(string.to_string());
            }
        }
    }
    
    name_strings.into_iter().collect::<Vec<_>>().join(" ")
}

/// Create a string representation of all supported Unicode codepoints
fn create_charset_string(font: &FontRef) -> String {
    let charmap = font.charmap();
    let mut codepoints = Vec::new();
    
    // Collect all supported codepoints
    for codepoint in 0..0x10FFFF {
        if charmap.map(codepoint).is_some() {
            codepoints.push(codepoint);
        }
    }
    
    // Sort and deduplicate codepoints
    codepoints.sort();
    codepoints.dedup();
    
    // Convert to a string of characters
    codepoints.into_iter()
        .filter_map(std::char::from_u32)
        .collect()
} 