#!/bin/bash

# Release script for lock-rs
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if version is provided
if [ $# -eq 0 ]; then
    print_error "Usage: $0 <version>"
    print_info "Example: $0 v0.1.0"
    exit 1
fi

VERSION="$1"

# Validate version format
if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    print_error "Invalid version format. Use: v0.1.0"
    exit 1
fi

print_info "Creating release $VERSION..."

# Check if git is clean
if [ -n "$(git status --porcelain)" ]; then
    print_error "Working directory is not clean. Commit or stash changes first."
    exit 1
fi

# Check if tag already exists
if git tag -l | grep -q "^$VERSION$"; then
    print_error "Tag $VERSION already exists"
    exit 1
fi

# Update version in Cargo.toml
print_info "Updating version in Cargo.toml..."
sed -i "s/^version = \".*\"/version = \"${VERSION#v}\"/" Cargo.toml

# Commit version update
git add Cargo.toml
git commit -m "chore: bump version to $VERSION"

# Create and push tag
print_info "Creating tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"
git push origin main
git push origin "$VERSION"

print_success "Release $VERSION created successfully!"
print_info "GitHub Actions will now build and publish the release."
print_info "Check: https://github.com/elly-hacen/lock-rs/actions"
