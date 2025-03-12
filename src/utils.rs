// this_file: fontgrep/src/utils.rs
//
// Utility functions and helpers

use crate::{FontgrepError, Result};
use dirs::data_dir;
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Get the modification time of a file as seconds since the Unix epoch
pub fn get_file_mtime(path: &Path) -> Result<i64> {
    let metadata = fs::metadata(path)?;
    let mtime = metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| FontgrepError::Io(format!("Failed to get mtime: {}", e)))?
        .as_secs() as i64;

    Ok(mtime)
}

/// Get the size of a file in bytes
pub fn get_file_size(path: &Path) -> Result<i64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len() as i64)
}

/// Determine the cache path based on the provided path or default location
pub fn determine_cache_path(cache_path: Option<&str>) -> Result<PathBuf> {
    match cache_path {
        Some(":memory:") => Ok(PathBuf::from(":memory:")),
        Some(path) if !path.is_empty() => Ok(PathBuf::from(path)),
        _ => {
            // Use default location in user's data directory
            let data_dir = data_dir().ok_or_else(|| {
                FontgrepError::Cache("Could not determine data directory".to_string())
            })?;

            let fontgrep_dir = data_dir.join("fontgrep");
            fs::create_dir_all(&fontgrep_dir).map_err(|e| {
                FontgrepError::Io(format!("Failed to create cache directory: {}", e))
            })?;

            Ok(fontgrep_dir.join("cache.db"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_file_size() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        {
            let mut file = File::create(&file_path).unwrap();
            file.write_all(b"Hello, World!").unwrap();
        }

        let size = get_file_size(&file_path).unwrap();
        assert_eq!(size, 13);
    }

    #[test]
    fn test_determine_cache_path() {
        // Test with explicit path
        let explicit_path = determine_cache_path(Some("/tmp/test.db")).unwrap();
        assert_eq!(explicit_path, PathBuf::from("/tmp/test.db"));

        // Test with in-memory database
        let memory_path = determine_cache_path(Some(":memory:")).unwrap();
        assert_eq!(memory_path, PathBuf::from(":memory:"));

        // Test with empty path (should use default)
        let default_path = determine_cache_path(Some("")).unwrap();
        assert!(default_path.ends_with("fontgrep/cache.db"));

        // Test with None (should use default)
        let none_path = determine_cache_path(None).unwrap();
        assert!(none_path.ends_with("fontgrep/cache.db"));
    }
}
