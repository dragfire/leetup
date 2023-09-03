#!/bin/bash

# Get the desired version bump type from command line argument
if [ $# -ne 1 ]; then
  echo "Usage: $0 [major|minor|patch]"
  exit 1
fi

VERSION_BUMP="$1"

# Get the current version from git tags or set to 1.0.0 if no tags exist
CURRENT_VERSION=$(git describe --abbrev=0 --tags 2>/dev/null || echo "v1.0.0")
CURRENT_VERSION=${CURRENT_VERSION:1}

echo "Current Version: $CURRENT_VERSION"

# Split the current version into an array using '.' as a delimiter
IFS='.' read -ra CURRENT_VERSION_PARTS <<< "$CURRENT_VERSION"

# Extract major, minor, and patch version numbers
VNUM1=${CURRENT_VERSION_PARTS[0]}
VNUM2=${CURRENT_VERSION_PARTS[1]}
VNUM3=${CURRENT_VERSION_PARTS[2]}

# Increment the version based on the bump type
if [ "$VERSION_BUMP" == 'major' ]; then
  VNUM1=$((VNUM1+1))
  VNUM2=0
  VNUM3=0
elif [ "$VERSION_BUMP" == 'minor' ]; then
  VNUM2=$((VNUM2+1))
  VNUM3=0
elif [ "$VERSION_BUMP" == 'patch' ]; then
  VNUM3=$((VNUM3+1))
else
  echo "Invalid version bump type. Use 'major', 'minor', or 'patch'."
  exit 1
fi

# Create the new version number
NEW_TAG="v$VNUM1.$VNUM2.$VNUM3"
echo "Bumping $VERSION_BUMP version to $NEW_TAG"

# Create a new tag and push it to the remote repository
git tag "$NEW_TAG"
git push --tags
git push

exit 0

