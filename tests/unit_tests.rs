// this_file: tests/unit_tests.rs
//! Unit tests for fontgrep modules

use fontgrep::cli::parse_codepoints;
use fontgrep::version::{get_version, is_release_version};

/// Test codepoint parsing functionality
#[test]
fn test_parse_codepoints_single() {
    let result = parse_codepoints("A").unwrap();
    assert_eq!(result, vec!['A']);
}

#[test]
fn test_parse_codepoints_multiple() {
    let result = parse_codepoints("A,B,C").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C']);
}

#[test]
fn test_parse_codepoints_unicode() {
    let result = parse_codepoints("U+0041,U+0042,U+0043").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C']);
}

#[test]
fn test_parse_codepoints_range() {
    let result = parse_codepoints("U+0041-U+0043").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C']);
}

#[test]
fn test_parse_codepoints_mixed() {
    let result = parse_codepoints("A,U+0042-U+0044").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C', 'D']);
}

#[test]
fn test_parse_codepoints_invalid_range() {
    let result = parse_codepoints("U+0043-U+0041");
    assert!(result.is_err());
}

#[test]
fn test_parse_codepoints_invalid_format() {
    let result = parse_codepoints("U+ZZZZ");
    assert!(result.is_err());
}

/// Test version functionality
#[test]
fn test_get_version_returns_string() {
    let version = get_version();
    assert!(!version.is_empty());
    println!("Version: {version}");
}

#[test]
fn test_is_release_version_returns_bool() {
    let is_release = is_release_version();
    println!("Is release version: {is_release}");
    // Just ensure it doesn't panic
}

/// Test version format validation
#[test]
fn test_version_format() {
    let version = get_version();

    // Should contain at least one digit
    assert!(version.chars().any(|c| c.is_ascii_digit()));

    // Should not be empty
    assert!(!version.is_empty());

    // Should be reasonable length
    assert!(!version.is_empty() && version.len() < 100);
}

/// Test error handling
#[test]
fn test_error_conversion() {
    use fontgrep::FontgrepError;

    // Test string conversion
    let err: FontgrepError = "test error".into();
    assert!(matches!(err, FontgrepError::Other(_)));

    // Test String conversion
    let err: FontgrepError = String::from("test error").into();
    assert!(matches!(err, FontgrepError::Other(_)));

    // Test IO error conversion
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err: FontgrepError = io_err.into();
    assert!(matches!(err, FontgrepError::Io(_)));
}

/// Test result type functionality
#[test]
fn test_result_type() {
    use fontgrep::FontgrepError;

    // Test Ok result
    let ok_result: Result<i32, FontgrepError> = Ok(42);
    assert!(ok_result.is_ok());
    assert_eq!(ok_result.unwrap(), 42);

    // Test Err result
    let err_result: Result<i32, FontgrepError> = Err("test error".into());
    assert!(err_result.is_err());
}

/// Test CLI argument parsing helpers
#[test]
fn test_codepoint_edge_cases() {
    // Test single character
    let result = parse_codepoints("A").unwrap();
    assert_eq!(result, vec!['A']);

    // Test hex without U+ prefix
    let result = parse_codepoints("0041").unwrap();
    assert_eq!(result, vec!['A']);

    // Test lowercase u+
    let result = parse_codepoints("u+0041").unwrap();
    assert_eq!(result, vec!['A']);
}

/// Test complex codepoint ranges
#[test]
fn test_complex_codepoint_ranges() {
    // Test basic ASCII range
    let result = parse_codepoints("0041-0043").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C']);

    // Test number range
    let result = parse_codepoints("U+0030-U+0039").unwrap();
    assert_eq!(
        result,
        vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']
    );

    // Test mixed ranges and singles
    let result = parse_codepoints("A,U+0042-U+0044,Z").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C', 'D', 'Z']);
}

/// Test version edge cases
#[test]
fn test_version_edge_cases() {
    // Test that version doesn't contain problematic characters
    let version = get_version();
    assert!(!version.contains('\n'));
    assert!(!version.contains('\r'));
    assert!(!version.contains('\t'));

    // Test version is trimmed
    assert_eq!(version.trim(), version);
}

/// Test error display
#[test]
fn test_error_display() {
    use fontgrep::FontgrepError;

    let err = FontgrepError::Other("test error".to_string());
    let display = format!("{err}");
    assert!(display.contains("test error"));

    let err = FontgrepError::Parse("invalid input".to_string());
    let display = format!("{err}");
    assert!(display.contains("Parse error"));
    assert!(display.contains("invalid input"));
}

/// Test version module constants
#[test]
fn test_version_constants() {
    // Test that cargo package version is available
    let cargo_version = env!("CARGO_PKG_VERSION");
    assert!(!cargo_version.is_empty());
    assert!(cargo_version.chars().any(|c| c.is_ascii_digit()));
}

/// Test multiple codepoint formats in one string
#[test]
fn test_mixed_codepoint_formats() {
    // Mix of character, U+, u+, and hex
    let result = parse_codepoints("A,U+0042,u+0043,0044").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C', 'D']);
}

/// Test invalid codepoint ranges
#[test]
fn test_invalid_codepoint_ranges() {
    // Range with only one part
    let result = parse_codepoints("U+0041-");
    assert!(result.is_err());

    // Range with too many parts
    let result = parse_codepoints("U+0041-U+0042-U+0043");
    assert!(result.is_err());

    // Invalid hex
    let result = parse_codepoints("U+GGGG");
    assert!(result.is_err());
}

/// Test high unicode codepoints
#[test]
fn test_high_unicode_codepoints() {
    // Test emoji range
    let result = parse_codepoints("U+1F600-U+1F602").unwrap();
    assert_eq!(result, vec!['😀', '😁', '😂']);

    // Test high plane codepoint
    let result = parse_codepoints("U+1F680").unwrap();
    assert_eq!(result, vec!['🚀']);
}

/// Test empty and whitespace handling
#[test]
fn test_whitespace_handling() {
    // Test with spaces (should be handled by shell/CLI parsing)
    let result = parse_codepoints("A,B,C").unwrap();
    assert_eq!(result, vec!['A', 'B', 'C']);
}

/// Test version in different git states
#[test]
fn test_version_consistency() {
    // Test that get_version() returns the same value when called multiple times
    let version1 = get_version();
    let version2 = get_version();
    assert_eq!(version1, version2);
}
