#!/usr/bin/env bash
# Build Ciel-vX.Y.Z.skill from ciel.skill/
# Usage: ./scripts/build-skill.sh <version>
#   version: semver without leading 'v' (e.g. 1.2.3 or 1.2.3-rc.1)
# Outputs to dist/:
#   Ciel-vX.Y.Z.skill          (ZIP archive; the installable bundle)
#   Ciel-vX.Y.Z.skill.sha256   (SHA-256 of the .skill file)
#   SHA256SUMS                 (aggregate; also includes component files)

set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "usage: $0 <version>" >&2
  exit 2
fi

# Normalize: strip any leading 'v'
VERSION="${VERSION#v}"

# Validate semver (loose; accepts pre-release + build metadata)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "error: version '$VERSION' is not valid semver" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/ciel.skill"
DIST="$ROOT/dist"
ARTIFACT="Ciel-v${VERSION}.skill"

if [[ ! -d "$SRC" ]]; then
  echo "error: source dir not found: $SRC" >&2
  exit 1
fi

# Cross-check the tag version against SKILL.md + MANIFEST.md frontmatter / body
# Permits release scripts to catch version drift before publishing.
decl_skill=""
if grep -q '^version:' "$SRC/SKILL.md"; then
  decl_skill="$(awk -F': *' '/^version:/{print $2; exit}' "$SRC/SKILL.md" | tr -d '[:space:]')"
fi
decl_manifest=""
if grep -Eq '^- \*\*Version\*\*:' "$SRC/MANIFEST.md"; then
  decl_manifest="$(awk -F'`' '/^- \*\*Version\*\*:/{print $2; exit}' "$SRC/MANIFEST.md")"
fi

if [[ -n "$decl_skill" && "$decl_skill" != "$VERSION" ]]; then
  echo "error: SKILL.md declares version '$decl_skill' but build requested '$VERSION'" >&2
  exit 3
fi
if [[ -n "$decl_manifest" && "$decl_manifest" != "$VERSION" ]]; then
  echo "warn: MANIFEST.md declares version '$decl_manifest' but build requested '$VERSION'" >&2
fi

echo "==> Building $ARTIFACT"
mkdir -p "$DIST"
rm -f "$DIST/$ARTIFACT" "$DIST/${ARTIFACT}.sha256" "$DIST/SHA256SUMS"

# Stage a clean copy (exclude OS / editor cruft).
STAGE="$(mktemp -d -t ciel-build.XXXXXX)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/ciel.skill"
cp -r "$SRC/." "$STAGE/ciel.skill/"

# Clean up excluded patterns manually since cp -r doesn't support --exclude
find "$STAGE/ciel.skill" -type f \( -name ".DS_Store" -o -name "Thumbs.db" -o -name "*.swp" -o -name "*~" -o -name "*.bak" -o -name "*.orig" \) -delete
find "$STAGE/ciel.skill" -type d \( -name ".idea" -o -name ".vscode" \) -exec rm -rf {} +

# Ensure install scripts retain exec bit.
chmod +x "$STAGE/ciel.skill/init/scripts/install.sh" "$STAGE/ciel.skill/init/scripts/verify.sh"

# Deterministic ZIP: sort entries. Pipe file list to zip's -@ stdin mode so
# the archive contains exactly those paths (no directory entries).
pushd "$STAGE/ciel.skill" >/dev/null
find . -type f | LC_ALL=C sort >"$STAGE/file-list"

if command -v zip >/dev/null 2>&1; then
  zip -X -q -@ "$STAGE/$ARTIFACT" <"$STAGE/file-list"
else
  echo "error: 'zip' not found" >&2
  exit 1
fi
popd >/dev/null

mv "$STAGE/$ARTIFACT" "$DIST/$ARTIFACT"

# Hashes
pushd "$DIST" >/dev/null
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum "$ARTIFACT" >"${ARTIFACT}.sha256"
  sha256sum "$ARTIFACT" >SHA256SUMS
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$ARTIFACT" >"${ARTIFACT}.sha256"
  shasum -a 256 "$ARTIFACT" >SHA256SUMS
else
  echo "error: no sha256 tool available" >&2
  exit 1
fi
popd >/dev/null

# Summary
ls -l "$DIST/$ARTIFACT" "$DIST/${ARTIFACT}.sha256"
echo
echo "==> Build complete: $DIST/$ARTIFACT"
