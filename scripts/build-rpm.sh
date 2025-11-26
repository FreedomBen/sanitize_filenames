#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

BINARY_NAME="sanitize_filenames"
VERSION="$(awk -F\" '/^version = / {print $2; exit}' Cargo.toml)"

if [ -z "${VERSION}" ]; then
  echo "Error: could not determine version from Cargo.toml" >&2
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "Error: git is required to build the RPM source archive" >&2
  exit 1
fi

ARCHIVE="${BINARY_NAME}-${VERSION}.tar.gz"

git -C "${ROOT_DIR}" ls-files -z | \
  tar --null -T - \
      --transform="s,^,${BINARY_NAME}-${VERSION}/," \
      -czf "${ARCHIVE}"

if [ ! -f "${ARCHIVE}" ]; then
  echo "Error: expected archive ${ARCHIVE} was not created" >&2
  exit 1
fi

for dist in fc41 fc42 fc43 el8 el9 el10; do
  echo "Building RPM for dist .${dist}..."
  rpmbuild -ba \
    --define "_sourcedir ${ROOT_DIR}" \
    --define "_srcrpmdir ${ROOT_DIR}/target/srpm" \
    --define "_rpmdir ${ROOT_DIR}/target/rpm/${dist}" \
    --define "_builddir ${ROOT_DIR}/target/rpmbuild/${dist}" \
    --define "dist .${dist}" \
    sanitize_filenames.spec
done

rm -f "${BINARY_NAME}-${VERSION}.tar.gz"
