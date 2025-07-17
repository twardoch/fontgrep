// this_file: tests/integration_tests.rs
//! Integration tests for fontgrep

use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

/// Test basic CLI functionality
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("finds fonts that"));
    assert!(stdout.contains("--features"));
    assert!(stdout.contains("--axes"));
    assert!(stdout.contains("--scripts"));
}

/// Test version output
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("fontgrep"));
    // Should contain version number
    assert!(stdout.chars().any(|c| c.is_ascii_digit()));
}

/// Test search with testdata
#[test]
fn test_search_testdata() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping testdata test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--", testdata_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // Should find some fonts
    assert!(!stdout.is_empty());
}

/// Test JSON output
#[test]
fn test_json_output() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping JSON test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--", "-j", testdata_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should be valid JSON
    if !stdout.trim().is_empty() {
        let parsed: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(parsed.is_array());
    }
}

/// Test variable fonts search
#[test]
fn test_variable_fonts_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping variable fonts test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--", "--variable", testdata_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should succeed even if no variable fonts found
    assert!(output.status.success());
}

/// Test feature search
#[test]
fn test_feature_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping feature search test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-f",
            "liga,kern",
            testdata_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should succeed even if no fonts with features found
    assert!(output.status.success());
}

/// Test script search
#[test]
fn test_script_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping script search test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--", "-s", "latn", testdata_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should succeed - most fonts support Latin
    assert!(output.status.success());
}

/// Test axes search
#[test]
fn test_axes_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping axes search test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-a",
            "wght,wdth",
            testdata_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should succeed even if no fonts with axes found
    assert!(output.status.success());
}

/// Test name pattern search
#[test]
fn test_name_pattern_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping name pattern test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-n",
            "Regular",
            testdata_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should succeed - many fonts have "Regular" in name
    assert!(output.status.success());
}

/// Test codepoints search
#[test]
fn test_codepoints_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping codepoints test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-u",
            "U+0041-U+005A",
            testdata_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should succeed - most fonts support A-Z
    assert!(output.status.success());
}

/// Test text search
#[test]
fn test_text_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping text search test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--", "-t", "Hello", testdata_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should succeed - most fonts support "Hello"
    assert!(output.status.success());
}

/// Test tables search
#[test]
fn test_tables_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping tables test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-T",
            "GPOS,GSUB",
            testdata_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Should succeed even if no fonts with tables found
    assert!(output.status.success());
}

/// Test verbose output
#[test]
fn test_verbose_output() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping verbose test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args(["run", "--", "--verbose", testdata_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
}

/// Test with non-existent directory
#[test]
fn test_non_existent_directory() {
    let output = Command::new("cargo")
        .args(["run", "--", "/non/existent/path"])
        .output()
        .expect("Failed to execute command");

    // Should succeed but output warning
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("does not exist"));
}

/// Test with empty directory
#[test]
fn test_empty_directory() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(["run", "--", temp_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should succeed but find no fonts
    assert!(output.status.success());
}

/// Test combining multiple search criteria
#[test]
fn test_combined_search() {
    let testdata_path = PathBuf::from("testdata");
    if !testdata_path.exists() {
        eprintln!("Skipping combined search test - testdata directory not found");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "-j",
            "--variable",
            "-s",
            "latn",
            "-t",
            "Hello",
            testdata_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should be valid JSON
    if !stdout.trim().is_empty() {
        let parsed: serde_json::Value =
            serde_json::from_str(&stdout).expect("Output should be valid JSON");
        assert!(parsed.is_array());
    }
}
