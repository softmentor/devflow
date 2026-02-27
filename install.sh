#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${DWF_INSTALL_DIR:-${HOME}/.local/bin}"
TARGET_BIN="${INSTALL_DIR}/dwf"
PATH_EXPORT="export PATH=\"${INSTALL_DIR}:\$PATH\""

log() {
  printf '[install] %s\n' "$*"
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'Error: required command not found: %s\n' "$1" >&2
    exit 1
  fi
}

path_contains_install_dir() {
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) return 0 ;;
    *) return 1 ;;
  esac
}

append_path_export_if_missing() {
  local profile_file="$1"

  touch "${profile_file}"
  if ! grep -F "${PATH_EXPORT}" "${profile_file}" >/dev/null 2>&1; then
    {
      printf '\n# Added by devflow install.sh\n'
      printf '%s\n' "${PATH_EXPORT}"
    } >>"${profile_file}"
    log "updated ${profile_file} with ${INSTALL_DIR}"
  else
    log "${profile_file} already contains PATH entry"
  fi
}

source_profile_file() {
  local profile_file="$1"
  local sourced=false

  case "${profile_file}" in
    "${HOME}/.zshrc")
      if [[ -n "${ZSH_VERSION:-}" ]]; then
        sourced=true
      fi
      ;;
    "${HOME}/.bashrc"| "${HOME}/.bash_profile")
      if [[ -n "${BASH_VERSION:-}" ]]; then
        sourced=true
      fi
      ;;
  esac

  if [[ "${sourced}" != true ]]; then
    log "skipping source ${profile_file} from incompatible shell process"
    return
  fi

  # shellcheck disable=SC1090
  set +u
  source "${profile_file}"
  set -u
  log "sourced ${profile_file}"
}

is_script_sourced() {
  [[ "${BASH_SOURCE[0]}" != "$0" ]]
}

ensure_path_persisted() {
  local profile_file=""

  if path_contains_install_dir; then
    log "PATH already includes ${INSTALL_DIR}"
    return
  fi

  case "${SHELL:-}" in
    */zsh)
      profile_file="${HOME}/.zshrc"
      ;;
    */bash)
      if [[ -f "${HOME}/.bashrc" ]]; then
        profile_file="${HOME}/.bashrc"
      else
        profile_file="${HOME}/.bash_profile"
      fi
      ;;
    *)
      log "could not detect bash/zsh profile; add this line manually:"
      log "${PATH_EXPORT}"
      ;;
  esac

  if [[ -n "${profile_file}" ]]; then
    append_path_export_if_missing "${profile_file}"
    export PATH="${INSTALL_DIR}:${PATH}"
    source_profile_file "${profile_file}"
    if is_script_sourced; then
      log "PATH updated in current shell session"
    else
      log "current parent shell cannot be changed from this process"
      log "run: source ${profile_file} (or open a new terminal)"
    fi
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
ensure_path_persisted

log "done"
log "binary: ${TARGET_BIN}"
