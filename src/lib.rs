// Main library entry point for fontgrep
use thiserror::Error;

/// Error type for fontgrep
#[derive(Error, Debug)]
pub enum FontgrepError {
    /// IO errors
    #[error("IO error: {0}")]
    Io(String),

    /// Font parsing errors
    #[error("Font error: {0}")]
    Font(String),

    /// Parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

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
type Result<T> = std::result::Result<T, FontgrepError>;

pub mod cli;
mod font;
mod query;

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
}
