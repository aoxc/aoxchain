#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

# AOXChain lightweight self-healing supervisor for non-container local deployments.
# It restarts the producer loop when it exits unexpectedly.

resolve_bin_path() {
  if [[ -n "${BIN_PATH:-}" && -x "${BIN_PATH}" ]]; then
    printf "%s" "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${HOME}/.aoxc/bin/aoxc" ]]; then
    printf "%s" "${HOME}/.aoxc/bin/aoxc"
    return 0
  fi

  if [[ -x "./bin/aoxc" ]]; then
    printf "%s" "./bin/aoxc"
    return 0
  fi

  return 1
}

BIN_PATH="$(resolve_bin_path || true)"
MAX_RESTARTS="${MAX_RESTARTS:-20}"
RESTART_DELAY_SECS="${RESTART_DELAY_SECS:-3}"

if [[ -z "${BIN_PATH}" || ! -x "${BIN_PATH}" ]]; then
  echo "[error] binary is not executable: ${BIN_PATH}" >&2
  echo "Build/install it with: make package-bin" >&2
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
