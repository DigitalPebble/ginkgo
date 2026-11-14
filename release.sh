#!/bin/bash
set -euo pipefail

# Releases ginkgo without pushing to main directly, so that main can be protected:
#   1. opens a pull request on a release-<version> branch which sets the version in Cargo.toml
#   2. once that PR is merged, tags the merge commit on main (v<version>) and pushes the tag
# The tag triggers .github/workflows/release.yml, which builds the binaries
# and publishes them with their SHA-256 checksums as GitHub release assets.

command -v gh >/dev/null || { echo "The GitHub CLI (gh) is required"; exit 1; }
command -v cargo >/dev/null || { echo "cargo is required"; exit 1; }

current_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1
}

if [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  echo "Tracked files have uncommitted changes, aborting."
  exit 1
fi

git fetch origin main --tags
git checkout main
git merge --ff-only origin/main

CURRENT_VERSION=$(current_version)
echo "Current version in Cargo.toml: $CURRENT_VERSION"
echo ""

# Suggest the next patch version
MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3 | sed 's/[^0-9].*//')
SUGGESTED="$MAJOR.$MINOR.$((PATCH + 1))"

read -rp "Release version [$SUGGESTED]: " RELEASE_VERSION
RELEASE_VERSION=${RELEASE_VERSION:-$SUGGESTED}
# Accept "v0.3.0" as well: Cargo.toml holds 0.3.0, the git tag is always v0.3.0
RELEASE_VERSION=${RELEASE_VERSION#v}

if ! [[ "$RELEASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+].*)?$ ]]; then
  echo "'$RELEASE_VERSION' is not a valid version (expected X.Y.Z)"
  exit 1
fi
TAG="v$RELEASE_VERSION"

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
  echo "Tag $TAG already exists"
  exit 1
fi

BRANCH="release-$RELEASE_VERSION"

echo ""
echo "========================================="
echo "  Release version : $RELEASE_VERSION"
echo "  Git tag         : $TAG"
echo "  Branch / PR     : $BRANCH"
echo "========================================="
read -rp "Proceed? [y/N]: " CONFIRM
[[ "$CONFIRM" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }

echo ""
echo "--- Running checks ---"
cargo clippy --all-targets -- -Dwarnings
cargo test

echo "--- Setting version to $RELEASE_VERSION on branch $BRANCH ---"
git checkout -b "$BRANCH"
awk -v v="$RELEASE_VERSION" '!done && /^version = "/ {print "version = \"" v "\""; done=1; next} {print}' Cargo.toml > Cargo.toml.new
mv Cargo.toml.new Cargo.toml
cargo check --quiet   # refreshes Cargo.lock

git add Cargo.toml
if git ls-files --error-unmatch Cargo.lock >/dev/null 2>&1; then
  git add Cargo.lock
fi
git commit -sm "Release $RELEASE_VERSION"
git push -u origin "$BRANCH"

echo "--- Opening pull request ---"
gh pr create --base main --head "$BRANCH" \
  --title "Release $RELEASE_VERSION" \
  --body "Sets the version to $RELEASE_VERSION. Once merged, release.sh tags main to publish the release."

echo ""
echo "Merge the pull request (reviews/checks permitting), then come back here."
until [ "$(gh pr view "$BRANCH" --json state --jq .state)" = "MERGED" ]; do
  read -rp "Press Enter to re-check the PR status (Ctrl-C to stop): " _
done

echo "--- Tagging main ---"
git checkout main
git pull --ff-only origin main
if [ "$(current_version)" != "$RELEASE_VERSION" ]; then
  echo "Cargo.toml on main is at $(current_version), expected $RELEASE_VERSION"
  exit 1
fi
git tag -a "$TAG" -m "Release $RELEASE_VERSION"
git push origin "$TAG"

git branch -d "$BRANCH" 2>/dev/null || true
git push origin --delete "$BRANCH" 2>/dev/null || true

echo ""
echo "Done! Tag $TAG pushed."
echo "The release workflow is now building the binaries:"
gh repo view --json url --jq '.url + "/actions/workflows/release.yml"'
