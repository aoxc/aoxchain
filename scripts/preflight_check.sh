#!/usr/bin/env bash
set -Eeuo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

usage() {
  cat <<'OUT'
Usage: scripts/preflight_check.sh [--dry-run] [--verbose]
Runs operator preflight checks via `make ops-doctor`.
OUT
  print_common_flag_help
}

args=("$@")
if [[ " ${args[*]} " == *" --help "* ]]; then usage; exit 0; fi
parse_common_flags args
[[ ${#args[@]} -eq 0 ]] || die "Unknown argument: ${args[*]}" 2

run_make_target ops-doctor
