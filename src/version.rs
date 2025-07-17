// this_file: src/version.rs
//! Version management for fontgrep using git tags

/// Get the current version from git tags (determined at build time)
pub fn get_version() -> String {
    // Get version from build script
    let git_version = env!("GIT_VERSION");
    let git_hash = env!("GIT_HASH");
    let is_dirty = env!("GIT_DIRTY") == "true";

    if git_version.ends_with("-dev") {
        // Development version
        if is_dirty {
            format!("{git_version}-{git_hash}-dirty")
        } else {
            format!("{git_version}-{git_hash}")
        }
    } else {
        // Release version
        git_version.to_string()
    }
}

/// Check if current version is a release version (has exact git tag)
pub fn is_release_version() -> bool {
    let git_version = env!("GIT_VERSION");
    !git_version.ends_with("-dev")
}

/// Get the git hash
pub fn get_git_hash() -> String {
    env!("GIT_HASH").to_string()
}

/// Check if the working directory is dirty
pub fn is_git_dirty() -> bool {
    env!("GIT_DIRTY") == "true"
}

/// Get build information
pub fn get_build_info() -> String {
    format!(
        "fontgrep {} ({}{})",
        get_version(),
        get_git_hash(),
        if is_git_dirty() { "-dirty" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version_returns_string() {
        let version = get_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_is_release_version_returns_bool() {
        // This should not panic
        let _is_release = is_release_version();
    }
}
