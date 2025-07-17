#!/bin/bash
# this_file: scripts/release.sh
# Release script for fontgrep

set -euo pipefail

# Function to show usage
usage() {
    echo "Usage: $0 [patch|minor|major|<version>]"
    echo ""
    echo "Examples:"
    echo "  $0 patch       # Increment patch version (0.1.0 -> 0.1.1)"
    echo "  $0 minor       # Increment minor version (0.1.0 -> 0.2.0)"
    echo "  $0 major       # Increment major version (0.1.0 -> 1.0.0)"
    echo "  $0 1.2.3       # Set specific version"
    exit 1
}

# Check if argument provided
if [ $# -eq 0 ]; then
    usage
fi

VERSION_TYPE=$1

# Get current version
CURRENT_VERSION=$(grep "^version = " Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "📝 Current version: $CURRENT_VERSION"

# Calculate new version
case $VERSION_TYPE in
    patch)
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{$NF = $NF + 1; print}' OFS='.')
        ;;
    minor)
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{$(NF-1) = $(NF-1) + 1; $NF = 0; print}' OFS='.')
        ;;
    major)
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{$1 = $1 + 1; $(NF-1) = 0; $NF = 0; print}' OFS='.')
        ;;
    *)
        if [[ $VERSION_TYPE =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
            NEW_VERSION=$VERSION_TYPE
        else
            echo "❌ Invalid version format: $VERSION_TYPE"
            usage
        fi
        ;;
esac

echo "🎯 New version: $NEW_VERSION"

# Confirm release
read -p "🚀 Create release $NEW_VERSION? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Release cancelled"
    exit 1
fi

# Update version in Cargo.toml
echo "📝 Updating Cargo.toml version..."
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Run full build and test
echo "🔧 Running full build and test..."
./scripts/build.sh
./scripts/test.sh

# Update changelog
echo "📋 Updating CHANGELOG.md..."
DATE=$(date +%Y-%m-%d)
if [ -f "CHANGELOG.md" ]; then
    # Add new version to changelog
    sed -i "2i\\## [$NEW_VERSION] - $DATE\\n\\n### Added\\n- TBD: Add release notes\\n\\n### Changed\\n- TBD: Add changes\\n\\n### Fixed\\n- TBD: Add fixes\\n" CHANGELOG.md
else
    # Create new changelog
    cat > CHANGELOG.md << EOF
# Changelog

## [$NEW_VERSION] - $DATE

### Added
- TBD: Add release notes

### Changed
- TBD: Add changes

### Fixed
- TBD: Add fixes

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
EOF
fi

echo "📝 Please edit CHANGELOG.md to add release notes"
echo "🎯 Then run: git add . && git commit -m 'Release $NEW_VERSION' && git tag v$NEW_VERSION"
echo "🚀 Finally: git push origin main && git push origin v$NEW_VERSION"

echo "✅ Release preparation complete!"
echo "🏷️  Tagged version: v$NEW_VERSION"