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

PKGREL=1
PKGNAME="sanitize-filenames"
ARCH="x86_64"
PKGVER="${VERSION}-${PKGREL}"

# Build the binary if it does not already exist
if [ ! -x "target/${TARGET}/release/${BINARY_NAME}" ]; then
  cargo build --release --target "${TARGET}"
fi

PKG_ROOT="target/archpkg/${PKGNAME}-${PKGVER}"
PKG_DEST_DIR="target/archpkg"

rm -rf "${PKG_ROOT}"
mkdir -p "${PKG_ROOT}/usr/bin"
mkdir -p "${PKG_ROOT}/usr/share/doc/${PKGNAME}"
mkdir -p "${PKG_ROOT}/usr/share/licenses/${PKGNAME}"
mkdir -p "${PKG_ROOT}/usr/share/bash-completion/completions"
mkdir -p "${PKG_ROOT}/usr/share/zsh/site-functions"
mkdir -p "${PKG_ROOT}/usr/share/fish/vendor_completions.d"

install -m 0755 "target/${TARGET}/release/${BINARY_NAME}" \
  "${PKG_ROOT}/usr/bin/${BINARY_NAME}"

install -m 0644 LICENSE \
  "${PKG_ROOT}/usr/share/licenses/${PKGNAME}/LICENSE"

install -m 0644 README.md \
  "${PKG_ROOT}/usr/share/doc/${PKGNAME}/README.md"

install -m 0644 completions/sanitize_filenames.bash \
  "${PKG_ROOT}/usr/share/bash-completion/completions/sanitize_filenames"

install -m 0644 completions/_sanitize_filenames \
  "${PKG_ROOT}/usr/share/zsh/site-functions/_sanitize_filenames"

install -m 0644 completions/sanitize_filenames.fish \
  "${PKG_ROOT}/usr/share/fish/vendor_completions.d/sanitize_filenames.fish"

SIZE="$(du -sb "${PKG_ROOT}" | cut -f1 || echo 0)"
BUILD_DATE="$(date +%s)"

PKGINFO="${PKG_ROOT}/.PKGINFO"
cat > "${PKGINFO}" <<EOF
pkgname = ${PKGNAME}
pkgver = ${PKGVER}
pkgdesc = CLI tool to sanitize filenames
url = https://example.com/sanitize_filenames
builddate = ${BUILD_DATE}
packager = Unknown <unknown@example.com>
size = ${SIZE}
arch = ${ARCH}
license = AGPL3
EOF

mkdir -p "${PKG_DEST_DIR}"
OUTPUT_PKG="${PKG_DEST_DIR}/${PKGNAME}-${PKGVER}-${ARCH}.pkg.tar.zst"

tar -C "${PKG_ROOT}" -c . | zstd -q -o "${OUTPUT_PKG}"

echo "Built Arch package: ${OUTPUT_PKG}"
