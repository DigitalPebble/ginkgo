#!/usr/bin/env bash
# Downloads and verifies the prebuilt ginkgo binary matching the runner.
# Writes `bin=<path>` to $GITHUB_OUTPUT, or an empty `bin=` when the action
# should build from source instead (no release for this ref, unsupported runner).
set -euo pipefail

REPO="${ACTION_REPOSITORY:-DigitalPebble/ginkgo}"
VERSION="${INPUT_VERSION:-}"

set_output() {
  echo "bin=$1" >> "${GITHUB_OUTPUT:-/dev/stdout}"
}

# Pinned to a release tag (uses: DigitalPebble/ginkgo@v0.3.0) => matching binary.
if [ -z "$VERSION" ] && [[ "${ACTION_REF:-}" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
  VERSION="$ACTION_REF"
fi

if [ -z "$VERSION" ]; then
  echo "No release version requested (ref '${ACTION_REF:-local}'), building from source"
  set_output ""
  exit 0
fi

if [ "$VERSION" = "latest" ]; then
  auth=()
  [ -n "${GITHUB_TOKEN:-}" ] && auth=(-H "Authorization: Bearer $GITHUB_TOKEN")
  VERSION=$(curl -fsSL "${auth[@]}" "https://api.github.com/repos/$REPO/releases/latest" \
    | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
  [ -n "$VERSION" ] || { echo "::error::Could not determine the latest release of $REPO"; exit 1; }
fi

case "${RUNNER_OS:-}-${RUNNER_ARCH:-}" in
  Linux-X64)    TARGET=x86_64-unknown-linux-gnu;  EXT=tar.gz; EXE=ginkgo ;;
  Linux-ARM64)  TARGET=aarch64-unknown-linux-gnu; EXT=tar.gz; EXE=ginkgo ;;
  macOS-ARM64)  TARGET=aarch64-apple-darwin;      EXT=tar.gz; EXE=ginkgo ;;
  Windows-X64)  TARGET=x86_64-pc-windows-msvc;    EXT=zip;    EXE=ginkgo.exe ;;
  *)
    echo "::warning::No prebuilt binary for ${RUNNER_OS:-?}/${RUNNER_ARCH:-?}, building from source"
    set_output ""
    exit 0
    ;;
esac

ASSET="ginkgo-$VERSION-$TARGET.$EXT"
URL="https://github.com/$REPO/releases/download/$VERSION"
DEST="${RUNNER_TEMP:-/tmp}/ginkgo-$VERSION"
mkdir -p "$DEST"

echo "Downloading $ASSET"
curl -fsSL --retry 3 -o "$DEST/$ASSET" "$URL/$ASSET" \
  || { echo "::error::Could not download $URL/$ASSET"; exit 1; }
curl -fsSL --retry 3 -o "$DEST/$ASSET.sha256" "$URL/ginkgo-$VERSION-$TARGET.sha256" \
  || { echo "::error::Could not download the checksum for $ASSET"; exit 1; }

EXPECTED=$(cut -d' ' -f1 "$DEST/$ASSET.sha256")
if command -v sha256sum >/dev/null; then
  ACTUAL=$(sha256sum "$DEST/$ASSET" | cut -d' ' -f1)
else
  ACTUAL=$(shasum -a 256 "$DEST/$ASSET" | cut -d' ' -f1)
fi
if [ -z "$EXPECTED" ] || [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "::error::Checksum mismatch for $ASSET (expected '$EXPECTED', got '$ACTUAL')"
  exit 1
fi
echo "Checksum OK ($ACTUAL)"

if [ "$EXT" = "zip" ]; then
  unzip -q -o "$DEST/$ASSET" -d "$DEST" 2>/dev/null || 7z x -y -o"$DEST" "$DEST/$ASSET" >/dev/null
else
  tar -xzf "$DEST/$ASSET" -C "$DEST"
fi
chmod +x "$DEST/$EXE"

set_output "$DEST/$EXE"
