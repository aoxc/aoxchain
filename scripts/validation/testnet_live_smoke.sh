#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"

NODE_HOME="${NODE_HOME:-}"
STRICT_DIAGNOSTICS="${STRICT_DIAGNOSTICS:-0}"

log() {
  printf '[testnet-live-smoke] %s\n' "$*"
}

die() {
  printf '[testnet-live-smoke][error] %s\n' "$*" >&2
  exit 2
}

run() {
  local label="$1"
  shift
  log "running: ${label}"
  "$@"
}

[[ -n "${NODE_HOME}" ]] || die "NODE_HOME must be set (example: /mnt/xdbx/aoxc/aoxc-rolling-testnet-7n/nodes/node01/home)"
[[ -d "${NODE_HOME}" ]] || die "NODE_HOME does not exist: ${NODE_HOME}"

cd "${REPO_ROOT}"

run "node status" aoxc node status --home "${NODE_HOME}"
run "network status" aoxc network status --home "${NODE_HOME}"
run "network identity gate (testnet)" aoxc network-identity-gate --enforce --env testnet --home "${NODE_HOME}" --format json
run "genesis validate strict" aoxc genesis-validate --strict --home "${NODE_HOME}"
run "genesis production gate" aoxc genesis-production-gate --home "${NODE_HOME}"
run "testnet readiness enforce" aoxc testnet-readiness --enforce --home "${NODE_HOME}" --format json

if [[ "${STRICT_DIAGNOSTICS}" == "1" ]]; then
  run "diagnostics doctor (strict)" aoxc diagnostics-doctor --home "${NODE_HOME}"
else
  log "running: diagnostics doctor (advisory)"
  if ! aoxc diagnostics-doctor --home "${NODE_HOME}"; then
    log "diagnostics-doctor reported non-zero exit (advisory mode). Continuing."
  fi
fi

log "live smoke completed"
