#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

RUNTIME_ROOT="${AOXC_RUNTIME_ROOT:-${HOME}/.aoxc/runtime}"
SNAPSHOT_ROOT="${AOXC_RUNTIME_SNAPSHOTS_DIR:-${RUNTIME_ROOT}/snapshots}"
KEEP_COUNT="${AOXC_SNAPSHOT_KEEP:-12}"

DB_DIR="${RUNTIME_ROOT}/db"
STATE_DIR="${RUNTIME_ROOT}/state"
CONFIG_DIR="${RUNTIME_ROOT}/config"
IDENTITY_DIR="${RUNTIME_ROOT}/identity"
OPERATOR_DIR="${RUNTIME_ROOT}/operator"

usage() {
  cat <<USAGE
Usage: $(basename "$0") <command>

Commands:
  snapshot        Create timestamped snapshot archives for runtime directories.
  list            List available snapshots under snapshot root.
  restore-latest  Restore runtime directories from the newest snapshot.
  prune           Keep only newest N snapshots (AOXC_SNAPSHOT_KEEP, default: ${KEEP_COUNT}).
USAGE
}

require_safe_root() {
  if [[ -z "${RUNTIME_ROOT}" || "${RUNTIME_ROOT}" = "/" ]]; then
    echo "Unsafe AOXC runtime root: '${RUNTIME_ROOT}'" >&2
    exit 2
  fi
}

ensure_dirs() {
  mkdir -p "${RUNTIME_ROOT}" "${SNAPSHOT_ROOT}"
}

ts_now() {
  TZ=UTC date +%Y%m%dT%H%M%SZ
}

create_component_archive() {
  local name="$1"
  local src="$2"
  local target_dir="$3"

  if [[ -d "${src}" ]]; then
    tar -C "${RUNTIME_ROOT}" -czf "${target_dir}/${name}.tar.gz" "$(basename "${src}")"
    echo "component=${name} status=ok source=${src}" >> "${target_dir}/manifest.txt"
  else
    echo "component=${name} status=missing source=${src}" >> "${target_dir}/manifest.txt"
  fi
}

snapshot() {
  require_safe_root
  ensure_dirs

  local stamp
  stamp="$(ts_now)"
  local out_dir="${SNAPSHOT_ROOT}/${stamp}"

  mkdir -p "${out_dir}"
  {
    echo "timestamp_utc=${stamp}"
    echo "runtime_root=${RUNTIME_ROOT}"
    echo "snapshot_root=${SNAPSHOT_ROOT}"
  } > "${out_dir}/manifest.txt"

  create_component_archive "db" "${DB_DIR}" "${out_dir}"
  create_component_archive "state" "${STATE_DIR}" "${out_dir}"
  create_component_archive "config" "${CONFIG_DIR}" "${out_dir}"
  create_component_archive "identity" "${IDENTITY_DIR}" "${out_dir}"
  create_component_archive "operator" "${OPERATOR_DIR}" "${out_dir}"

  echo "snapshot_created=${out_dir}"
}

list_snapshots() {
  require_safe_root
  ensure_dirs

  find "${SNAPSHOT_ROOT}" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort
}

latest_snapshot_dir() {
  find "${SNAPSHOT_ROOT}" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort | tail -n 1
}

restore_component_archive() {
  local name="$1"
  local target_dir="$2"
  local archive="$3"

  rm -rf "${target_dir}"
  mkdir -p "${RUNTIME_ROOT}"
  tar -C "${RUNTIME_ROOT}" -xzf "${archive}"
  echo "restored_component=${name} archive=${archive}"
}

restore_latest() {
  require_safe_root
  ensure_dirs

  local latest
  latest="$(latest_snapshot_dir)"

  if [[ -z "${latest}" ]]; then
    echo "No snapshot found under ${SNAPSHOT_ROOT}" >&2
    exit 3
  fi

  local base="${SNAPSHOT_ROOT}/${latest}"
  [[ -f "${base}/manifest.txt" ]] || { echo "Missing manifest in snapshot: ${base}" >&2; exit 3; }

  [[ -f "${base}/db.tar.gz" ]] && restore_component_archive "db" "${DB_DIR}" "${base}/db.tar.gz"
  [[ -f "${base}/state.tar.gz" ]] && restore_component_archive "state" "${STATE_DIR}" "${base}/state.tar.gz"
  [[ -f "${base}/config.tar.gz" ]] && restore_component_archive "config" "${CONFIG_DIR}" "${base}/config.tar.gz"
  [[ -f "${base}/identity.tar.gz" ]] && restore_component_archive "identity" "${IDENTITY_DIR}" "${base}/identity.tar.gz"
  [[ -f "${base}/operator.tar.gz" ]] && restore_component_archive "operator" "${OPERATOR_DIR}" "${base}/operator.tar.gz"

  echo "restored_from=${base}"
}

prune_snapshots() {
  require_safe_root
  ensure_dirs

  if ! [[ "${KEEP_COUNT}" =~ ^[0-9]+$ ]] || [[ "${KEEP_COUNT}" -lt 1 ]]; then
    echo "AOXC_SNAPSHOT_KEEP must be a positive integer, got: ${KEEP_COUNT}" >&2
    exit 2
  fi

  mapfile -t dirs < <(find "${SNAPSHOT_ROOT}" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort)
  local total="${#dirs[@]}"

  if (( total <= KEEP_COUNT )); then
    echo "prune_skipped=1 total=${total} keep=${KEEP_COUNT}"
    return 0
  fi

  local remove_count=$(( total - KEEP_COUNT ))
  local i
  for (( i=0; i<remove_count; i++ )); do
    rm -rf "${SNAPSHOT_ROOT}/${dirs[$i]}"
    echo "pruned_snapshot=${SNAPSHOT_ROOT}/${dirs[$i]}"
  done
}

main() {
  local command="${1:-}"
  case "${command}" in
    snapshot)
      snapshot
      ;;
    list)
      list_snapshots
      ;;
    restore-latest)
      restore_latest
      ;;
    prune)
      prune_snapshots
      ;;
    -h|--help|help)
      usage
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
}

main "$@"
