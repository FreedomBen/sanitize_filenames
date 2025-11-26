#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

if command -v podman >/dev/null 2>&1; then
  RUNTIME=podman
elif command -v docker >/dev/null 2>&1; then
  RUNTIME=docker
else
  echo "Error: neither podman nor docker is installed or in PATH" >&2
  exit 1
fi

if [ "${RUNTIME}" = "podman" ]; then
  VOLUME_OPT="-v ${ROOT_DIR}:/workspace:Z"
else
  VOLUME_OPT="-v ${ROOT_DIR}:/workspace"
fi

IMAGE_NAME="sanitize-filenames-static-binary"

"${RUNTIME}" build -f "${ROOT_DIR}/containerfiles/Containerfile.static-binary" -t "${IMAGE_NAME}" "${ROOT_DIR}"

"${RUNTIME}" run --rm \
  ${VOLUME_OPT} \
  -w /workspace \
  "${IMAGE_NAME}" \
  make build
