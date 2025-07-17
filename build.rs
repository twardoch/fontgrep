// this_file: build.rs
//! Build script for fontgrep

use std::process::Command;

fn main() {
    // Get git information at build time
    let git_version = get_git_version();
    let git_hash = get_git_hash();
    let is_dirty = is_git_dirty();

    // Pass git information to the build
    println!("cargo:rustc-env=GIT_VERSION={git_version}");
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=GIT_DIRTY={is_dirty}");

    // Rebuild if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads/main");
    println!("cargo:rerun-if-changed=.git/refs/tags/");
}

fn get_git_version() -> String {
    // Try to get version from git tag first
    if let Ok(output) = Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(stripped) = version.strip_prefix('v') {
                return stripped.to_string();
            }
            return version;
        }
    }

    // Fallback to git describe
    if let Ok(output) = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Some(stripped) = version.strip_prefix('v') {
                return format!("{stripped}-dev");
            }
            return format!("{version}-dev");
        }
    }

    // Final fallback to cargo version
    env!("CARGO_PKG_VERSION").to_string()
}

fn get_git_hash() -> String {
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}

fn is_git_dirty() -> bool {
    if let Ok(output) = Command::new("git").args(["status", "--porcelain"]).output() {
        if output.status.success() {
            return !output.stdout.is_empty();
        }
    }
    false
}
