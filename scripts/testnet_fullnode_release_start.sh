#!/usr/bin/env bash
# AOXC MIT License
# Persistent testnet full-node launcher for release-layout deployments.
set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_RELEASES_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

HOME_DIR="/mnt/xdbx/.aoxc-data/testnet-fullnode-01"
DATA_ROOT="/mnt/xdbx/.aoxc-data"
RELEASES_ROOT="${DEFAULT_RELEASES_ROOT}"
PROFILE="testnet"
NETWORK_KIND="testnet"
RELEASE_VERSION=""
RELEASE_PLATFORM=""
AOXC_BIN_OVERRIDE=""
SKIP_BOOTSTRAP=0
NODE_NAME=""

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Persistent testnet full-node launcher for release bundles.
Optimized for release roots such as /mnt/xdbx/aoxc with binaries under:
  v<version>/<platform>/bin/aoxc

Options:
  --home <path>                 Node home path (default: ${HOME_DIR})
  --data-root <path>            Base data root for node state (default: ${DATA_ROOT})
  --releases-root <path>        Release root path (default: ${RELEASES_ROOT})
  --version <value>             Release version, e.g. 0.2.0-alpha.1
  --release-version <value>     Alias of --version
  --platform <value>            Release platform, e.g. linux-amd64
  --release-platform <value>    Alias of --platform
  --aoxc-bin <path>             Explicit aoxc binary path override
  --profile <name>              Bootstrap profile (default: ${PROFILE})
  --network-kind <kind>         Runtime source network kind (default: ${NETWORK_KIND})
  --name <value>                Node name passed to production-bootstrap (default: basename of --home)
  --skip-bootstrap              Skip production-bootstrap and run only node start
  -h, --help                    Show this help
USAGE
}

log() { printf '[testnet-fullnode][info] %s\n' "$*"; }
fail() { printf '[testnet-fullnode][error] %s\n' "$*" >&2; exit 2; }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

ensure_parent_writable() {
  local target="$1"
  local parent
  parent="$(dirname "${target}")"
  while [[ ! -d "${parent}" && "${parent}" != "/" ]]; do
    parent="$(dirname "${parent}")"
  done
  [[ -d "${parent}" ]] || fail "Could not resolve existing parent directory for: ${target}"
  [[ -w "${parent}" ]] || fail "Parent directory is not writable: ${parent}"
}

resolve_default_platform() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m | tr '[:upper:]' '[:lower:]')"
  case "${arch}" in
    x86_64|amd64) arch="amd64" ;;
    aarch64|arm64) arch="arm64" ;;
  esac
  printf '%s-%s' "${os}" "${arch}"
}

resolve_release_version_from_root() {
  local matches
  matches="$(find "${RELEASES_ROOT}" -maxdepth 1 -mindepth 1 -type d -name 'v*' -printf '%f\n' | sort)"
  [[ -n "${matches}" ]] || fail "No release version directories found under: ${RELEASES_ROOT}"
  if [[ "$(printf '%s\n' "${matches}" | wc -l | tr -d ' ')" != "1" ]]; then
    fail "Multiple version directories found; provide --version explicitly"
  fi
  printf '%s' "${matches#v}"
}

read_workspace_version_if_repo() {
  local cargo_toml="$1/Cargo.toml"
  [[ -f "${cargo_toml}" ]] || return 0
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' "${cargo_toml}" | head -n1
}

read_policy_version_if_repo() {
  local policy="$1/configs/version-policy.toml"
  [[ -f "${policy}" ]] || return 0
  sed -n '/^\[workspace\]/,/^\[/{s/^current = "\(.*\)"/\1/p}' "${policy}" | head -n1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --home) HOME_DIR="$2"; shift 2 ;;
    --data-root) DATA_ROOT="$2"; shift 2 ;;
    --releases-root) RELEASES_ROOT="$2"; shift 2 ;;
    --version|--release-version) RELEASE_VERSION="$2"; shift 2 ;;
    --platform|--release-platform) RELEASE_PLATFORM="$2"; shift 2 ;;
    --aoxc-bin) AOXC_BIN_OVERRIDE="$2"; shift 2 ;;
    --profile) PROFILE="$2"; shift 2 ;;
    --network-kind) NETWORK_KIND="$2"; shift 2 ;;
    --name) NODE_NAME="$2"; shift 2 ;;
    --skip-bootstrap) SKIP_BOOTSTRAP=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Unknown argument: $1 (use --help for supported options)" ;;
  esac
done

require_cmd uname
require_cmd find
require_cmd sed

[[ -d "${RELEASES_ROOT}" ]] || fail "Release root does not exist: ${RELEASES_ROOT}"

if [[ -z "${RELEASE_VERSION}" ]]; then
  RELEASE_VERSION="$(resolve_release_version_from_root)"
fi

if [[ -z "${RELEASE_PLATFORM}" ]]; then
  RELEASE_PLATFORM="$(resolve_default_platform)"
fi

if [[ -z "${AOXC_BIN_OVERRIDE}" ]]; then
  AOXC_BIN="${RELEASES_ROOT%/}/v${RELEASE_VERSION}/${RELEASE_PLATFORM}/bin/aoxc"
else
  AOXC_BIN="${AOXC_BIN_OVERRIDE}"
fi

[[ -x "${AOXC_BIN}" ]] || fail "aoxc binary is not executable: ${AOXC_BIN}"

if [[ -z "${NODE_NAME}" ]]; then
  NODE_NAME="$(basename "${HOME_DIR}")"
fi

# Repository parity checks are enforced only when repository files are present.
REPO_ROOT_CANDIDATE="$(cd "${SCRIPT_DIR}/.." && pwd)"
WORKSPACE_VERSION="$(read_workspace_version_if_repo "${REPO_ROOT_CANDIDATE}")"
POLICY_VERSION="$(read_policy_version_if_repo "${REPO_ROOT_CANDIDATE}")"
if [[ -n "${WORKSPACE_VERSION}" && -n "${POLICY_VERSION}" ]]; then
  [[ "${WORKSPACE_VERSION}" == "${POLICY_VERSION}" ]] || fail "Version mismatch: Cargo.toml=${WORKSPACE_VERSION}, version-policy=${POLICY_VERSION}"
fi
if [[ -n "${WORKSPACE_VERSION}" ]]; then
  [[ "${RELEASE_VERSION}" == "${WORKSPACE_VERSION}" ]] || fail "--version (${RELEASE_VERSION}) does not match workspace version (${WORKSPACE_VERSION})"
fi

ensure_parent_writable "${HOME_DIR}"
mkdir -p "${HOME_DIR}" "${DATA_ROOT}"

BINARY_VERSION_OUTPUT="$(${AOXC_BIN} version 2>/dev/null || true)"
if [[ -n "${BINARY_VERSION_OUTPUT}" ]] && [[ "${BINARY_VERSION_OUTPUT}" != *"${RELEASE_VERSION}"* ]]; then
  fail "Binary version output does not include selected version '${RELEASE_VERSION}': ${BINARY_VERSION_OUTPUT}"
fi

log "Release root            : ${RELEASES_ROOT}"
log "Release version         : ${RELEASE_VERSION}"
log "Release platform        : ${RELEASE_PLATFORM}"
log "aoxc binary             : ${AOXC_BIN}"
log "Data root               : ${DATA_ROOT}"
log "Node home               : ${HOME_DIR}"
log "Profile                 : ${PROFILE}"
log "Network kind            : ${NETWORK_KIND}"
log "Node name               : ${NODE_NAME}"

if [[ "${SKIP_BOOTSTRAP}" -eq 0 ]]; then
  log "Running production-bootstrap"
  AOXC_HOME="${HOME_DIR}" AOXC_NETWORK_KIND="${NETWORK_KIND}" "${AOXC_BIN}" \
    production-bootstrap --profile "${PROFILE}" --name "${NODE_NAME}"
fi

log "Starting persistent full node"
exec env AOXC_HOME="${HOME_DIR}" AOXC_NETWORK_KIND="${NETWORK_KIND}" \
  "${AOXC_BIN}" node start --home "${HOME_DIR}"
