// this_file: fontgrep/src/lib.rs
//
// Main library entry point for fontgrep

use thiserror::Error;

/// Default batch size for database operations
pub const DEFAULT_BATCH_SIZE: usize = 100;

/// Error type for fontgrep
#[derive(Error, Debug)]
pub enum FontgrepError {
    /// IO errors
    #[error("IO error: {0}")]
    Io(String),

    /// Font parsing errors
    #[error("Font error: {0}")]
    Font(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Cache errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// Parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Connection pool errors
    #[error("Connection pool error: {0}")]
    Pool(#[from] r2d2::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Regex errors
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Memory mapping errors
    #[error("Memory mapping error: {0}")]
    Mmap(String),

    /// Font loading errors
    #[error("Font loading error: {0}")]
    FontLoad(String),

    /// Font feature errors
    #[error("Font feature error: {0}")]
    Feature(String),

    /// Font script errors
    #[error("Font script error: {0}")]
    Script(String),

    /// Font table errors
    #[error("Font table error: {0}")]
    Table(String),

    /// Font name errors
    #[error("Font name error: {0}")]
    Name(String),

    /// Font charset errors
    #[error("Font charset error: {0}")]
    Charset(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Other errors
    #[error("Error: {0}")]
    Other(String),
}

/// Result type for fontgrep
pub type Result<T> = std::result::Result<T, FontgrepError>;

// Re-export modules
pub mod cache;
pub mod cli;
pub mod font;
pub mod query;
pub mod utils;

// Re-export important types
pub use cache::FontCache;
pub use font::FontInfo;
pub use query::{FontQuery, QueryCriteria};

// Implement From for common error types
impl From<std::io::Error> for FontgrepError {
    fn from(err: std::io::Error) -> Self {
        FontgrepError::Io(format!("{:?}: {}", err.kind(), err))
    }
}

impl From<String> for FontgrepError {
    fn from(err: String) -> Self {
        FontgrepError::Other(err)
    }
}

impl From<&str> for FontgrepError {
    fn from(err: &str) -> Self {
        FontgrepError::Other(err.to_string())
    }
}

/// Helper function to add context to errors
pub fn with_context<T, C>(result: Result<T>, context: C) -> Result<T>
where
    C: FnOnce() -> String,
{
    result.map_err(|e| {
        let ctx = context();
        FontgrepError::Other(format!("{}: {}", ctx, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let fontgrep_err: FontgrepError = io_err.into();

        match fontgrep_err {
            FontgrepError::Io(msg) => {
                assert!(msg.contains("file not found"));
                assert!(msg.contains("NotFound"));
            }
            _ => panic!("Expected Io error"),
        }

        let str_err = "test error";
        let fontgrep_err: FontgrepError = str_err.into();

        match fontgrep_err {
            FontgrepError::Other(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Other error"),
        }
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(FontgrepError::Io("file not found".to_string()));
        let with_ctx = with_context(result, || "Failed to open font file".to_string());

        match with_ctx {
            Err(FontgrepError::Other(msg)) => {
                assert!(msg.contains("Failed to open font file"));
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected error with context"),
        }
    }
}
