#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

usage() {
  echo "Usage: $(basename "$0") <version>" >&2
  echo "  Example versions: v1.1.1, 1.1.1, v0.1.0, 2.2.2" >&2
  exit 1
}

if [[ $# -ne 1 ]]; then
  usage
fi

RAW_VERSION="$1"
VERSION="${RAW_VERSION#v}"

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: version must be in the form X.Y.Z (optionally prefixed with 'v')" >&2
  exit 1
fi

MAN_DATE="$(date '+%B %Y')"
SPEC_DATE="$(date '+%a %b %e %Y')"

echo "Bumping version to $VERSION"
echo "Man page date: $MAN_DATE"
echo "Spec changelog date: $SPEC_DATE"

cd "$ROOT_DIR"

echo "Updating Cargo.toml..."
sed -i.bak -E 's/^version = "[0-9]+\.[0-9]+\.[0-9]+"/version = "'"$VERSION"'"/' Cargo.toml
rm -f Cargo.toml.bak

if [[ -f Cargo.lock ]]; then
  echo "Updating Cargo.lock..."
  awk 'BEGIN{pkg=0}
    /^\[\[package\]\]/{pkg=0}
    /^name = "sanitize_filenames"/{pkg=1}
    pkg && /^version = /{$0="version = \"'"$VERSION"'\""; pkg=0}
    {print}' Cargo.lock > Cargo.lock.tmp && mv Cargo.lock.tmp Cargo.lock
fi

if [[ -f PKGBUILD ]]; then
  echo "Updating PKGBUILD..."
  sed -i.bak -E \
    -e 's/^pkgver=.*/pkgver='"$VERSION"'/g' \
    -e 's/^pkgrel=.*/pkgrel=1/g' \
    PKGBUILD
  rm -f PKGBUILD.bak
fi

if [[ -f sanitize_filenames.spec ]]; then
  echo "Updating sanitize_filenames.spec..."
  sed -i.bak -E \
    -e 's/^(Version:[[:space:]]*)[0-9]+\.[0-9]+\.[0-9]+/\1'"$VERSION"'/g' \
    sanitize_filenames.spec

  # Replace the first changelog line (the most recent entry).
  sed -i -E "0,/^\* .*/s//\* $SPEC_DATE Packager Name <packager@example.com> - $VERSION-1/" \
    sanitize_filenames.spec

  rm -f sanitize_filenames.spec.bak
fi

if [[ -f man/sanitize_filenames.1 ]]; then
  echo "Updating man/sanitize_filenames.1..."
  sed -i.bak -E \
    's/^(\\.TH SANITIZE_FILENAMES 1 \")([^\"]+)(\" \"sanitize_filenames )[0-9]+\.[0-9]+\.[0-9]+(\" \"User Commands\")/\\1'"$MAN_DATE"'\\3'"$VERSION"'\\4/' \
    man/sanitize_filenames.1
  rm -f man/sanitize_filenames.1.bak
fi

echo "Version bump to $VERSION complete."

