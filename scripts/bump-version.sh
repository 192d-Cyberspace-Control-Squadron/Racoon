#!/bin/bash
# Version bumping script for Racoon NOS
# Automatically bumps version based on commit type (feat = minor, fix = patch)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
current_version=$(grep -m1 'version = ' Cargo.toml | sed 's/.*version = "\(.*\)".*/\1/')

if [ -z "$current_version" ]; then
    echo -e "${RED}❌ Could not determine current version${NC}"
    exit 1
fi

echo -e "${GREEN}Current version: ${current_version}${NC}"

# Parse version components
IFS='.' read -r -a version_parts <<< "$current_version"
major="${version_parts[0]}"
minor="${version_parts[1]}"
patch="${version_parts[2]}"

# Get last commit message
last_commit=$(git log -1 --pretty=%B)

# Determine version bump type
if echo "$last_commit" | grep -qE "^feat(\(.+\))?: "; then
    # Feature commit - bump minor version
    minor=$((minor + 1))
    patch=0
    bump_type="minor"
    echo -e "${YELLOW}🎉 Feature detected - bumping minor version${NC}"
elif echo "$last_commit" | grep -qE "^fix(\(.+\))?: "; then
    # Fix commit - bump patch version
    patch=$((patch + 1))
    bump_type="patch"
    echo -e "${YELLOW}🔧 Fix detected - bumping patch version${NC}"
else
    echo -e "${YELLOW}ℹ️  No version bump needed (commit type: ${last_commit%%:*})${NC}"
    exit 0
fi

new_version="${major}.${minor}.${patch}"
echo -e "${GREEN}New version: ${new_version}${NC}"

# Update Cargo.toml
echo "📝 Updating Cargo.toml..."
sed -i.bak "s/version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
echo "🔒 Updating Cargo.lock..."
cargo check --quiet 2>/dev/null || true

# Update CHANGELOG.md
echo "📋 Updating CHANGELOG.md..."
today=$(date +%Y-%m-%d)

# Move unreleased changes to new version section
if grep -q "## \[Unreleased\]" CHANGELOG.md; then
    # Create new version section
    sed -i.bak "/## \[Unreleased\]/a\\
\\
## [${new_version}] - ${today}
" CHANGELOG.md
    rm CHANGELOG.md.bak
fi

# Stage changes
git add Cargo.toml Cargo.lock CHANGELOG.md

echo -e "${GREEN}✅ Version bumped from ${current_version} to ${new_version}${NC}"
echo -e "${YELLOW}📝 Files staged for commit. Run 'git commit' to finalize.${NC}"
