#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${DWF_INSTALL_DIR:-${HOME}/.local/bin}"
TARGET_BIN="${INSTALL_DIR}/dwf"

log() {
  printf '[install] %s\n' "$*"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Error: required command not found: %s\n' "$1" >&2
    exit 1
  fi
}

log "checking prerequisites"
require_cmd cargo
require_cmd rustc

log "building dwf (release)"
cargo build --release -p devflow-cli --manifest-path "${ROOT_DIR}/Cargo.toml"

log "installing to ${TARGET_BIN}"
mkdir -p "${INSTALL_DIR}"
cp "${ROOT_DIR}/target/release/dwf" "${TARGET_BIN}"
chmod +x "${TARGET_BIN}"

log "verifying installation"
"${TARGET_BIN}" --help >/dev/null

log "done"
log "binary: ${TARGET_BIN}"
log "if needed, add ${INSTALL_DIR} to your PATH"
