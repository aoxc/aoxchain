#!/usr/bin/env bash
set -euo pipefail

# AOXChain lightweight self-healing supervisor for non-container local deployments.
# It restarts the producer loop when it exits unexpectedly.

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
  echo "[supervisor] starting producer loop via produce-once"
  set +e
  for i in $(seq 1 1000000000); do
    "${BIN_PATH}" produce-once --tx "AOXC_SUPERVISOR_${i}"
    cmd_exit=$?
    if [[ ${cmd_exit} -ne 0 ]]; then
      exit_code=${cmd_exit}
      break
    fi
    sleep 1
  done
  if [[ -z "${exit_code:-}" ]]; then
    exit_code=0
  fi
  set -e

  if [[ $exit_code -eq 0 ]]; then
    echo "[supervisor] producer loop exited normally (code 0), stopping supervisor"
    exit 0
  fi

  restart_count=$((restart_count + 1))
  echo "[supervisor] producer loop failed with code ${exit_code}; restart ${restart_count}/${MAX_RESTARTS} in ${RESTART_DELAY_SECS}s"

  if [[ $restart_count -ge $MAX_RESTARTS ]]; then
    echo "[supervisor] max restart threshold reached, giving up" >&2
    exit 1
  fi

  unset exit_code
  sleep "${RESTART_DELAY_SECS}"
done
