#!/usr/bin/env bash
set -euo pipefail

# AOXChain lightweight self-healing supervisor for non-container local deployments.
# It restarts the node process when it exits unexpectedly.

BIN_PATH="${BIN_PATH:-./bin/aoxc}"
MAX_RESTARTS="${MAX_RESTARTS:-20}"
RESTART_DELAY_SECS="${RESTART_DELAY_SECS:-3}"

if [[ ! -x "${BIN_PATH}" ]]; then
  echo "[error] binary is not executable: ${BIN_PATH}" >&2
  echo "Build it with: make package-bin" >&2
  exit 2
fi

restart_count=0
while true; do
  echo "[supervisor] starting node: ${BIN_PATH} node"
  set +e
  "${BIN_PATH}" node
  exit_code=$?
  set -e

  if [[ $exit_code -eq 0 ]]; then
    echo "[supervisor] node exited normally (code 0), stopping supervisor"
    exit 0
  fi

  restart_count=$((restart_count + 1))
  echo "[supervisor] node crashed with code ${exit_code}; restart ${restart_count}/${MAX_RESTARTS} in ${RESTART_DELAY_SECS}s"

  if [[ $restart_count -ge $MAX_RESTARTS ]]; then
    echo "[supervisor] max restart threshold reached, giving up" >&2
    exit 1
  fi

  sleep "${RESTART_DELAY_SECS}"
done
