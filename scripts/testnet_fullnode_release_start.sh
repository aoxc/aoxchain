#!/usr/bin/env bash
# AOXC MIT License
# Build-and-run helper for a real testnet full node with release/version parity checks.
set -euo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

HOME_DIR=""
RELEASES_ROOT="/aoxchain/releases"
PROFILE="testnet"
NETWORK_KIND="testnet"
FORCE_REBUILD=0
SKIP_BUILD=0
RUN_BOOTSTRAP=1

usage() {
  cat <<USAGE
Usage: $(basename "$0") --home <path> [options]

Starts an AOXC full node for real testnet operation and enforces release-version parity.

Required:
  --home <path>                 Full-node home path (passed to --home)

Options:
  --releases-root <path>        Release root path (default: ${RELEASES_ROOT})
  --profile <name>              Bootstrap profile (default: ${PROFILE})
  --network-kind <kind>         Runtime source network kind (default: ${NETWORK_KIND})
  --skip-bootstrap              Skip production-bootstrap and only start node
  --skip-build                  Do not build if release binary is missing
  --force-rebuild               Always rebuild and replace release binary
  -h, --help                    Show this help
USAGE
}

log() {
  printf '[testnet-fullnode][info] %s\n' "$*"
}

fail() {
  printf '[testnet-fullnode][error] %s\n' "$*" >&2
  exit 2
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

read_workspace_version() {
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' "${REPO_ROOT}/Cargo.toml" | head -n1
}

read_version_policy_current() {
  sed -n '/^\[workspace\]/,/^\[/{s/^current = "\(.*\)"/\1/p}' "${REPO_ROOT}/configs/version-policy.toml" | head -n1
}

build_aoxc_release() {
  log "Building aoxc release binary with locked dependency graph"
  (
    cd "${REPO_ROOT}"
    cargo build --release --locked -p aoxcmd --bin aoxc
  )
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --home) HOME_DIR="$2"; shift 2 ;;
    --releases-root) RELEASES_ROOT="$2"; shift 2 ;;
    --profile) PROFILE="$2"; shift 2 ;;
    --network-kind) NETWORK_KIND="$2"; shift 2 ;;
    --skip-bootstrap) RUN_BOOTSTRAP=0; shift ;;
    --skip-build) SKIP_BUILD=1; shift ;;
    --force-rebuild) FORCE_REBUILD=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Unknown argument: $1" ;;
  esac
done

[[ -n "${HOME_DIR}" ]] || { usage; fail "--home is required"; }

require_cmd cargo
require_cmd sed

WORKSPACE_VERSION="$(read_workspace_version)"
POLICY_VERSION="$(read_version_policy_current)"
[[ -n "${WORKSPACE_VERSION}" ]] || fail "Unable to resolve workspace version from Cargo.toml"
[[ -n "${POLICY_VERSION}" ]] || fail "Unable to resolve policy current version from configs/version-policy.toml"
[[ "${WORKSPACE_VERSION}" == "${POLICY_VERSION}" ]] || fail "Version mismatch: Cargo.toml=${WORKSPACE_VERSION}, version-policy=${POLICY_VERSION}"

RELEASE_DIR="${RELEASES_ROOT%/}/v${WORKSPACE_VERSION}"
RELEASE_BIN_DIR="${RELEASE_DIR}/bin"
RELEASE_BIN="${RELEASE_BIN_DIR}/aoxc"
TARGET_BIN="${REPO_ROOT}/target/release/aoxc"

mkdir -p "${RELEASE_BIN_DIR}" "${HOME_DIR}"

if [[ "${FORCE_REBUILD}" -eq 1 ]]; then
  build_aoxc_release
  cp "${TARGET_BIN}" "${RELEASE_BIN}"
  chmod +x "${RELEASE_BIN}"
elif [[ ! -x "${RELEASE_BIN}" ]]; then
  if [[ "${SKIP_BUILD}" -eq 1 ]]; then
    fail "Release binary is missing and --skip-build was set: ${RELEASE_BIN}"
  fi
  build_aoxc_release
  cp "${TARGET_BIN}" "${RELEASE_BIN}"
  chmod +x "${RELEASE_BIN}"
fi

RELEASE_VERSION_OUTPUT="$(${RELEASE_BIN} version 2>/dev/null || true)"
if [[ -n "${RELEASE_VERSION_OUTPUT}" ]] && [[ "${RELEASE_VERSION_OUTPUT}" != *"${WORKSPACE_VERSION}"* ]]; then
  fail "Release binary version output does not include workspace version '${WORKSPACE_VERSION}': ${RELEASE_VERSION_OUTPUT}"
fi

log "Release binary           : ${RELEASE_BIN}"
log "Workspace version        : ${WORKSPACE_VERSION}"
log "Version-policy current   : ${POLICY_VERSION}"
log "Home                     : ${HOME_DIR}"
log "Profile                  : ${PROFILE}"
log "Network kind             : ${NETWORK_KIND}"

if [[ "${RUN_BOOTSTRAP}" -eq 1 ]]; then
  log "Running production-bootstrap"
  AOXC_HOME="${HOME_DIR}" AOXC_NETWORK_KIND="${NETWORK_KIND}" "${RELEASE_BIN}" \
    production-bootstrap --profile "${PROFILE}" --name "$(basename "${HOME_DIR}")"
fi

log "Starting full node"
AOXC_HOME="${HOME_DIR}" AOXC_NETWORK_KIND="${NETWORK_KIND}" "${RELEASE_BIN}" node start --home "${HOME_DIR}"
