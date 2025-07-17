# Release Process

This document describes the release process for fontgrep using the git-tag-based semantic versioning system.

## Overview

The release system automatically handles:
- Version management based on git tags
- Multi-platform binary builds
- GitHub release creation
- Crates.io publishing (when configured)

## Prerequisites

1. **Set up GitHub Actions workflows** following [WORKFLOWS.md](WORKFLOWS.md)
2. **Configure secrets** (optional):
   - `CRATES_TOKEN` for crates.io publishing

## Release Steps

### 1. Prepare for Release

```bash
# Make sure you're on the main branch with latest changes
git checkout main
git pull origin main

# Run tests to ensure everything is working
make test

# Run full CI pipeline locally
make ci
```

### 2. Create a Release

Use the provided release script:

```bash
# For patch release (0.1.0 -> 0.1.1)
make release patch

# For minor release (0.1.0 -> 0.2.0)
make release minor

# For major release (0.1.0 -> 1.0.0)
make release major

# For specific version
make release 1.2.3
```

### 3. Review and Edit Release Notes

The release script will:
1. Update `Cargo.toml` version
2. Update `CHANGELOG.md` with new version entry
3. Run build and test
4. Prepare git tag

**Important**: Edit `CHANGELOG.md` to add proper release notes before continuing.

### 4. Complete the Release

After editing the changelog:

```bash
# Commit changes
git add .
git commit -m "Release v1.2.3"

# Create and push tag
git tag v1.2.3
git push origin main
git push origin v1.2.3
```

### 5. Automated Process

Once the tag is pushed, GitHub Actions will automatically:

1. **Test the release** - Run full test suite
2. **Build binaries** for multiple platforms:
   - Linux x86_64
   - Linux x86_64 (musl)
   - Windows x86_64
   - macOS x86_64
   - macOS ARM64 (Apple Silicon)
3. **Create GitHub release** with binaries attached
4. **Publish to crates.io** (if configured)

## Version Format

The system uses semantic versioning with the following format:

- **Release versions**: `1.2.3` (from git tag `v1.2.3`)
- **Development versions**: `1.2.3-dev-abc1234` (includes commit hash)
- **Dirty development**: `1.2.3-dev-abc1234-dirty` (uncommitted changes)

## Manual Version Override

If you need to override the version manually:

```bash
# Edit Cargo.toml
sed -i 's/version = ".*"/version = "1.2.3"/' Cargo.toml

# Build and test
cargo build --release
cargo test

# Create tag
git add Cargo.toml
git commit -m "Bump version to 1.2.3"
git tag v1.2.3
git push origin main
git push origin v1.2.3
```

## Troubleshooting

### Release Build Fails

```bash
# Check build locally
make build

# Check tests
make test

# Check formatting and linting
make check
```

### Tag Already Exists

```bash
# Delete local tag
git tag -d v1.2.3

# Delete remote tag
git push origin --delete v1.2.3

# Create new tag
git tag v1.2.3
git push origin v1.2.3
```

### GitHub Actions Fails

1. Check the Actions tab in your GitHub repository
2. Look for error messages in the workflow logs
3. Common issues:
   - Missing secrets (CRATES_TOKEN)
   - Permission issues
   - Build failures on specific platforms

### Crates.io Publishing Fails

1. Ensure `CRATES_TOKEN` secret is set
2. Check that you have publishing permissions
3. Verify the version doesn't already exist

## Release Checklist

Before releasing:

- [ ] All tests pass locally (`make test`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Version is updated in `Cargo.toml`
- [ ] `CHANGELOG.md` has release notes
- [ ] Git tag matches version
- [ ] GitHub Actions workflows are set up

After releasing:

- [ ] GitHub release is created
- [ ] Binaries are attached to release
- [ ] Crates.io package is published (if configured)
- [ ] All CI checks pass
- [ ] Version is correctly reported by binary (`fontgrep --version`)

## Example Release Output

When you run `fontgrep --version`, you should see:

```bash
# For release version
fontgrep 1.2.3

# For development version
fontgrep 1.2.3-dev-abc1234

# For dirty development
fontgrep 1.2.3-dev-abc1234-dirty
```

This indicates the version system is working correctly.