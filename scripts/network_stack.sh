#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DAEMON_SCRIPT="${SCRIPT_DIR}/network_env_daemon.sh"
ENVIRONMENTS=(testnet mainnet)

usage() {
  echo "usage: $0 <start|once|status|stop|restart>" >&2
}

COMMAND="${1:-}"
if [[ -z "${COMMAND}" ]]; then
  usage
  exit 2
fi

run_for_all() {
  local cmd="${1:?missing-cmd}"
  local env
  for env in "${ENVIRONMENTS[@]}"; do
    echo "[network-stack] ${cmd} ${env}"
    "${DAEMON_SCRIPT}" "${cmd}" "${env}"
  done
}

start_dual() {
  local started=()
  local env

  for env in "${ENVIRONMENTS[@]}"; do
    echo "[network-stack] starting ${env}"
    if "${DAEMON_SCRIPT}" start "${env}"; then
      started+=("${env}")
    else
      echo "[network-stack][error] failed to start ${env}; rolling back started environments"
      local prev
      for prev in "${started[@]}"; do
        "${DAEMON_SCRIPT}" stop "${prev}" || true
      done
      exit 10
    fi
  done

  echo "[network-stack] dual stack is running (testnet + mainnet)"
}

case "${COMMAND}" in
  start)
    start_dual
    ;;
  once|status|stop)
    run_for_all "${COMMAND}"
    ;;
  restart)
    run_for_all stop
    start_dual
    ;;
  *)
    usage
    exit 3
    ;;
esac
