use crate::font::FontInfo;
use skrifa::Tag;
use std::collections::HashSet;

/// Trait for matching fonts
pub trait FontMatcher {
    /// Check if a font matches the criteria
    fn matches(&self, info: &FontInfo) -> bool;
}

/// Matcher for variation axes
pub(crate) struct AxesMatcher {
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
pub(crate) struct FeaturesMatcher {
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
pub(crate) struct ScriptsMatcher {
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
pub(crate) struct TablesMatcher {
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
pub(crate) struct VariableFontMatcher;

impl VariableFontMatcher {
    /// Create a new variable font matcher
    pub fn new() -> Self {
        Self
    }
}

impl Default for VariableFontMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FontMatcher for VariableFontMatcher {
    fn matches(&self, info: &FontInfo) -> bool {
        info.is_variable
    }
}

/// Matcher for Unicode codepoints
pub(crate) struct CodepointsMatcher {
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
pub(crate) struct NameMatcher {
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
