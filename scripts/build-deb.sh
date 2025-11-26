#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

BINARY_NAME="sanitize_filenames"
TARGET="x86_64-unknown-linux-musl"
VERSION="$(awk -F\" '/^version = / {print $2; exit}' Cargo.toml)"
if [ -z "${VERSION}" ]; then
  echo "Error: could not determine version from Cargo.toml" >&2
  exit 1
fi
DEB_PACKAGE="sanitize-filenames"
DEB_ARCH="amd64"

mkdir -p target/deb
rm -rf "target/deb/${DEB_PACKAGE}_${VERSION}"
mkdir -p "target/deb/${DEB_PACKAGE}_${VERSION}/DEBIAN"
mkdir -p "target/deb/${DEB_PACKAGE}_${VERSION}/usr/bin"
mkdir -p "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/doc/${DEB_PACKAGE}"
mkdir -p "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/bash-completion/completions"
mkdir -p "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/zsh/vendor-completions"
mkdir -p "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/fish/vendor_completions.d"

install -m 0755 "target/${TARGET}/release/${BINARY_NAME}" \
  "target/deb/${DEB_PACKAGE}_${VERSION}/usr/bin/${BINARY_NAME}"

cp LICENSE README.md \
  "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/doc/${DEB_PACKAGE}/"

install -m 0644 completions/sanitize_filenames.bash \
  "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/bash-completion/completions/sanitize_filenames"

install -m 0644 completions/_sanitize_filenames \
  "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/zsh/vendor-completions/_sanitize_filenames"

install -m 0644 completions/sanitize_filenames.fish \
  "target/deb/${DEB_PACKAGE}_${VERSION}/usr/share/fish/vendor_completions.d/sanitize_filenames.fish"

CONTROL_FILE="target/deb/${DEB_PACKAGE}_${VERSION}/DEBIAN/control"
{
  printf "Package: %s\n" "${DEB_PACKAGE}"
  printf "Version: %s-1\n" "${VERSION}"
  printf "Section: utils\n"
  printf "Priority: optional\n"
  printf "Architecture: %s\n" "${DEB_ARCH}"
  printf "Maintainer: Benjamin Porter <freedomben@protonmail.com>\n"
  printf "Description: CLI tool to sanitize filenames\n"
} > "${CONTROL_FILE}"

dpkg-deb --build "target/deb/${DEB_PACKAGE}_${VERSION}" \
  "target/deb/${DEB_PACKAGE}_${VERSION}_${DEB_ARCH}.deb"
