#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

BINARY_NAME="sanitize_filenames"
VERSION="$(sed -n 's/^version = \"\\(.*\\)\"/\\1/p' Cargo.toml)"

tar czf "${BINARY_NAME}-${VERSION}.tar.gz" \
  --transform="s,^,${BINARY_NAME}-${VERSION}/," \
  --exclude-vcs --exclude target --exclude '*.rpm' --exclude '*.tar.gz' . || [ $? -eq 1 ]

rpmbuild -ba \
  --define "_sourcedir ${ROOT_DIR}" \
  --define "_srcrpmdir ${ROOT_DIR}/target/srpm" \
  --define "_rpmdir ${ROOT_DIR}/target/rpm" \
  --define "_builddir ${ROOT_DIR}/target/rpmbuild" \
  sanitize_filenames.spec

rm -f "${BINARY_NAME}-${VERSION}.tar.gz"

