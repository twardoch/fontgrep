// this_file: fontgrep/src/font.rs
//
// Font information extraction and matching

use crate::{Result, FontgrepError};
use memmap2::Mmap;
use skrifa::FontRef;
use skrifa::prelude::*;
use skrifa::raw::TableProvider;
use std::{
    collections::{BTreeSet, HashSet},
    fs::File,
    path::Path,
};
use log;
use once_cell::sync::OnceCell;

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
    
    /// Charset as a BTreeSet of codepoints
    #[serde(skip)]
    charset: OnceCell<BTreeSet<u32>>,
    
    /// Charset string (lazily computed)
    #[serde(skip)]
    charset_string_cell: OnceCell<String>,
    
    /// Serializable charset string for persistence
    #[serde(rename = "charset_string")]
    charset_string_serialized: String,
}

impl FontInfo {
    /// Load font information from a file
    pub fn load(path: &Path) -> Result<Self> {
        let font = load_font(path)?;
        Self::from_font(&font)
    }
    
    /// Extract font information from a font reference
    pub fn from_font(font: &FontRef) -> Result<Self> {
        // Helper function to safely extract data with error handling
        fn extract_safely<T, F>(font_name: &str, extraction_fn: F) -> T
        where
            F: FnOnce() -> T + std::panic::UnwindSafe,
            T: Default,
        {
            match std::panic::catch_unwind(extraction_fn) {
                Ok(result) => result,
                Err(_) => {
                    log::warn!("Failed to extract data from font: {}", font_name);
                    T::default()
                }
            }
        }
        
        // Extract name string first since we need it for error messages
        let name_string = extract_name_string(font);
        log::debug!("Extracted font name: {}", name_string);
        
        // Extract all other properties with consistent error handling
        let is_variable = has_variations(font);
        log::debug!("Font is variable: {}", is_variable);
        
        let axes = extract_safely(&name_string, || extract_axes(font));
        log::debug!("Extracted {} variation axes", axes.len());
        
        let features = extract_safely(&name_string, || extract_features(font));
        log::debug!("Extracted {} OpenType features", features.len());
        
        let scripts = extract_safely(&name_string, || extract_scripts(font));
        log::debug!("Extracted {} OpenType scripts", scripts.len());
        
        let tables = extract_safely(&name_string, || extract_tables(font));
        log::debug!("Extracted {} font tables", tables.len());
        
        let charset = extract_safely(&name_string, || create_charset(font));
        let charset_string_serialized = charset_to_string(&charset);
        log::debug!("Extracted charset with {} codepoints", charset.len());
        
        // Create the FontInfo with lazy charset
        let info = FontInfo {
            name_string,
            is_variable,
            axes,
            features,
            scripts,
            tables,
            charset: OnceCell::new(),
            charset_string_cell: OnceCell::new(),
            charset_string_serialized,
        };
        
        // Initialize the charset
        let _ = info.charset.set(charset);
        
        Ok(info)
    }
    
    /// Get the charset string (lazily computed)
    pub fn charset_string(&self) -> &str {
        self.charset_string_cell.get_or_init(|| {
            // If we have a serialized charset string from deserialization, use that
            if !self.charset_string_serialized.is_empty() {
                return self.charset_string_serialized.clone();
            }
            
            // Otherwise, compute it from the charset
            if let Some(charset) = self.charset.get() {
                charset_to_string(charset)
            } else {
                String::new()
            }
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
    let path_str = path.to_string_lossy().to_string();
    let file = File::open(path)
        .map_err(|e| FontgrepError::Io(format!("Failed to open font file {}: {}", path_str, e)))?;
    
    let data = Box::leak(Box::new(unsafe { 
        Mmap::map(&file)
            .map_err(|e| FontgrepError::Io(format!("Failed to memory-map font file {}: {}", path_str, e)))?
    }));
    
    FontRef::new(data)
        .map_err(|e| FontgrepError::Font(format!("Failed to parse font data {}: {}", path_str, e)))
}

/// Check if a file is a font based on its extension
pub fn is_font_file(path: &Path) -> bool {
    FontInfo::is_font_file(path)
}

/// Create a charset from a font with optimized implementation
pub fn create_charset(font: &FontRef) -> BTreeSet<u32> {
    // Use HashSet for faster insertion
    let mut charset = HashSet::new();
    
    // Get the character map from the font
    let charmap = font.charmap();
    
    // If the font has a character map, extract all supported codepoints
    if charmap.has_map() {
        // Use the mappings() method to get all codepoint to glyph mappings
        // Reserve capacity for better performance
        charset.reserve(charmap.mappings().count());
        
        for (codepoint, _glyph_id) in charmap.mappings() {
            // Skip invalid Unicode codepoints
            if !is_invalid_unicode(codepoint) {
                charset.insert(codepoint);
            }
        }
    }
    
    // Convert HashSet to BTreeSet for sorted order
    // This is more efficient than inserting directly into a BTreeSet
    charset.into_iter().collect()
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
    
    // Fast path for common case: ASCII printable characters
    if (0x20..=0x7E).contains(&codepoint) {
        return false;
    }
    
    // Check for control characters (C0 and C1 controls, and DELETE)
    if codepoint <= 0x1F || codepoint == 0x7F || (0x80..=0x9F).contains(&codepoint) {
        return true;
    }
    
    // Check for surrogate pairs
    if (0xD800..=0xDFFF).contains(&codepoint) {
        return true;
    }
    
    // Check for noncharacters
    if (0xFDD0..=0xFDEF).contains(&codepoint) {
        return true;
    }
    
    // Check for noncharacters at the end of each plane
    if (codepoint & 0xFFFE) == 0xFFFE && codepoint <= 0x10FFFF {
        return true;
    }
    
    false
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
    font.axes().iter().map(|axis| axis.tag().to_string()).collect()
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
    font.table_directory.table_records().iter()
        .map(|record| record.tag().to_string())
        .collect()
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