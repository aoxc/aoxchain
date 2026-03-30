#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Provide a shell-native autonomy state control surface for AOXC operator and
#   orchestration workflows without requiring a Python runtime.
#
# Scope:
#   - Initialize deterministic file-backed autonomy state
#   - Persist desired and observed environment state
#   - Record operational events
#   - Record release metadata
#   - Print current status and recent event history in structured JSON form
#
# Storage Model:
#   - State root:      $AOXC_DATA_ROOT/state/autonomy
#   - Environment:     environment-state.json
#   - Event ledger:    events.jsonl
#   - Release ledger:  releases.jsonl
#
# Exit Codes:
#   0  Successful completion
#   1  General operational failure
#   2  Invalid invocation
#   3  Invalid configuration or unsupported environment
#   4  Missing required host dependency
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_NAME="$(basename "$0")"
readonly DEFAULT_DATA_ROOT="${HOME}/.AOXCData"
readonly STATE_SUBDIR="state/autonomy"
readonly ENV_STATE_FILE_NAME="environment-state.json"
readonly EVENTS_FILE_NAME="events.jsonl"
readonly RELEASES_FILE_NAME="releases.jsonl"
readonly DEFAULT_HISTORY_LIMIT=30
readonly MAX_HISTORY_LIMIT=200

AOXC_DATA_ROOT="${AOXC_DATA_ROOT:-$DEFAULT_DATA_ROOT}"
STATE_ROOT="${AOXC_DATA_ROOT}/${STATE_SUBDIR}"
ENV_STATE_FILE="${STATE_ROOT}/${ENV_STATE_FILE_NAME}"
EVENTS_FILE="${STATE_ROOT}/${EVENTS_FILE_NAME}"
RELEASES_FILE="${STATE_ROOT}/${RELEASES_FILE_NAME}"

log_error() {
  printf '[error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"
  log_error "${message}"
  exit "${exit_code}"
}

print_usage() {
  cat <<EOF
Usage:
  ${SCRIPT_NAME} <command> [options]

Commands:
  init
      Initialize autonomy state files.

  status
      Print autonomy state summary as JSON.

  set-env --env <mainnet|testnet|devnet> --desired-state <value> --observed-state <value> [--note <text>]
      Persist desired and observed environment control-plane state.

  event --env <mainnet|testnet|devnet> --action <value> --status <value> [--detail <text>]
      Append an operational event.

  release --version <tag> --artifact <path> [--evidence <path>]
      Record release metadata.

  history [--limit <1..200>]
      Print recent event history as JSON.
EOF
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 4
}

ensure_state_root() {
  if [[ -e "${STATE_ROOT}" && ! -d "${STATE_ROOT}" ]]; then
    die "State root exists but is not a directory: ${STATE_ROOT}" 3
  fi

  mkdir -p "${STATE_ROOT}" || die "Unable to create state root: ${STATE_ROOT}" 3
}

ensure_state_files() {
  ensure_state_root

  if [[ ! -f "${ENV_STATE_FILE}" ]]; then
    printf '{\n  "schema_version": 1,\n  "environments": []\n}\n' > "${ENV_STATE_FILE}"
  fi

  touch "${EVENTS_FILE}" "${RELEASES_FILE}"
}

now_utc() {
  date -u +"%Y-%m-%dT%H:%M:%SZ"
}

escape_json_string() {
  local raw="${1:-}"
  raw="${raw//\\/\\\\}"
  raw="${raw//\"/\\\"}"
  raw="${raw//$'\n'/\\n}"
  raw="${raw//$'\r'/\\r}"
  raw="${raw//$'\t'/\\t}"
  printf '%s' "${raw}"
}

normalize_env() {
  local env_value="${1:-}"
  case "${env_value}" in
    mainnet|testnet|devnet)
      printf '%s\n' "${env_value}"
      ;;
    *)
      die "Invalid environment '${env_value}'. Use mainnet|testnet|devnet." 3
      ;;
  esac
}

normalize_limit() {
  local limit_value="${1:-$DEFAULT_HISTORY_LIMIT}"

  [[ "${limit_value}" =~ ^[0-9]+$ ]] || die "Invalid history limit: ${limit_value}" 3

  if (( limit_value < 1 )); then
    limit_value=1
  fi

  if (( limit_value > MAX_HISTORY_LIMIT )); then
    limit_value="${MAX_HISTORY_LIMIT}"
  fi

  printf '%s\n' "${limit_value}"
}

json_array_from_jsonl_file() {
  local source_file="$1"
  local first=1
  local line=''

  printf '['

  if [[ -s "${source_file}" ]]; then
    while IFS= read -r line; do
      [[ -n "${line}" ]] || continue

      if (( first == 1 )); then
        first=0
      else
        printf ','
      fi

      printf '\n  %s' "${line}"
    done < "${source_file}"

    if (( first == 0 )); then
      printf '\n'
    fi
  fi

  printf ']'
}

cmd_init() {
  ensure_state_files

  cat <<EOF
{
  "status": "ok",
  "state_root": "$(escape_json_string "${STATE_ROOT}")",
  "initialized_at_utc": "$(now_utc)"
}
EOF
}

cmd_set_env() {
  local env=''
  local desired_state=''
  local observed_state=''
  local note=''

  while (( $# > 0 )); do
    case "$1" in
      --env)
        shift
        env="${1:-}"
        ;;
      --desired-state)
        shift
        desired_state="${1:-}"
        ;;
      --observed-state)
        shift
        observed_state="${1:-}"
        ;;
      --note)
        shift
        note="${1:-}"
        ;;
      *)
        die "Unknown argument for set-env: $1" 2
        ;;
    esac
    shift
  done

  [[ -n "${env}" ]] || die "Missing required argument: --env" 2
  [[ -n "${desired_state}" ]] || die "Missing required argument: --desired-state" 2
  [[ -n "${observed_state}" ]] || die "Missing required argument: --observed-state" 2

  env="$(normalize_env "${env}")"
  ensure_state_files

  awk \
    -v target_env="${env}" \
    -v desired_state="${desired_state}" \
    -v observed_state="${observed_state}" \
    -v updated_at_utc="$(now_utc)" \
    -v note="${note}" \
    '
    BEGIN {
      replaced = 0
      print "{"
      print "  \"schema_version\": 1,"
      print "  \"environments\": ["
    }
    /^  \{.*\}$/ {
      line = $0
      gsub(/^  /, "", line)

      if (line ~ "\"env\": \"" target_env "\"") {
        if (replaced == 1) {
          next
        }
        printf "    {\"env\":\"%s\",\"desired_state\":\"%s\",\"observed_state\":\"%s\",\"updated_at_utc\":\"%s\",\"note\":\"%s\"}",
               target_env, desired_state, observed_state, updated_at_utc, note
        replaced = 1
      } else {
        sub(/^/, "    ", line)
        printf "%s", line
      }

      print ","
      next
    }
    END {
      if (replaced == 0) {
        printf "    {\"env\":\"%s\",\"desired_state\":\"%s\",\"observed_state\":\"%s\",\"updated_at_utc\":\"%s\",\"note\":\"%s\"}\n",
               target_env, desired_state, observed_state, updated_at_utc, note
      }
      print "  ]"
      print "}"
    }' "${ENV_STATE_FILE}" | \
    sed '$!N;s/,\n  ]/\n  ]/;P;D' > "${ENV_STATE_FILE}.tmp"

  mv "${ENV_STATE_FILE}.tmp" "${ENV_STATE_FILE}"

  cat <<EOF
{
  "status": "ok",
  "env": "$(escape_json_string "${env}")",
  "desired_state": "$(escape_json_string "${desired_state}")",
  "observed_state": "$(escape_json_string "${observed_state}")"
}
EOF
}

cmd_event() {
  local env=''
  local action=''
  local status_value=''
  local detail=''

  while (( $# > 0 )); do
    case "$1" in
      --env)
        shift
        env="${1:-}"
        ;;
      --action)
        shift
        action="${1:-}"
        ;;
      --status)
        shift
        status_value="${1:-}"
        ;;
      --detail)
        shift
        detail="${1:-}"
        ;;
      *)
        die "Unknown argument for event: $1" 2
        ;;
    esac
    shift
  done

  [[ -n "${env}" ]] || die "Missing required argument: --env" 2
  [[ -n "${action}" ]] || die "Missing required argument: --action" 2
  [[ -n "${status_value}" ]] || die "Missing required argument: --status" 2

  env="$(normalize_env "${env}")"
  ensure_state_files

  printf '{"event_time_utc":"%s","env":"%s","action":"%s","status":"%s","detail":"%s"}\n' \
    "$(now_utc)" \
    "$(escape_json_string "${env}")" \
    "$(escape_json_string "${action}")" \
    "$(escape_json_string "${status_value}")" \
    "$(escape_json_string "${detail}")" >> "${EVENTS_FILE}"

  cat <<EOF
{
  "status": "ok",
  "event": {
    "env": "$(escape_json_string "${env}")",
    "action": "$(escape_json_string "${action}")",
    "result": "$(escape_json_string "${status_value}")"
  }
}
EOF
}

cmd_release() {
  local version=''
  local artifact=''
  local evidence=''

  while (( $# > 0 )); do
    case "$1" in
      --version)
        shift
        version="${1:-}"
        ;;
      --artifact)
        shift
        artifact="${1:-}"
        ;;
      --evidence)
        shift
        evidence="${1:-}"
        ;;
      *)
        die "Unknown argument for release: $1" 2
        ;;
    esac
    shift
  done

  [[ -n "${version}" ]] || die "Missing required argument: --version" 2
  [[ -n "${artifact}" ]] || die "Missing required argument: --artifact" 2

  ensure_state_files

  printf '{"release_time_utc":"%s","version_tag":"%s","artifact_path":"%s","evidence_path":"%s"}\n' \
    "$(now_utc)" \
    "$(escape_json_string "${version}")" \
    "$(escape_json_string "${artifact}")" \
    "$(escape_json_string "${evidence}")" >> "${RELEASES_FILE}"

  cat <<EOF
{
  "status": "ok",
  "release": {
    "version": "$(escape_json_string "${version}")",
    "artifact": "$(escape_json_string "${artifact}")"
  }
}
EOF
}

cmd_status() {
  local env_count=0
  local event_count=0
  local release_count=0

  ensure_state_files

  env_count="$(grep -c '"env":"' "${ENV_STATE_FILE}" || true)"
  event_count="$(grep -c '.*' "${EVENTS_FILE}" || true)"
  release_count="$(grep -c '.*' "${RELEASES_FILE}" || true)"

  cat <<EOF
{
  "status": "ok",
  "state_root": "$(escape_json_string "${STATE_ROOT}")",
  "environment_state": $(awk '
    BEGIN { in_array = 0; first = 1; printf "[" }
    /"environments": \[/ { in_array = 1; next }
    in_array && /^\]/ { in_array = 0; printf "]"; next }
    in_array {
      gsub(/^ +|,$/, "", $0)
      if (length($0) > 0) {
        if (first == 1) {
          first = 0
        } else {
          printf ","
        }
        printf "\n    %s", $0
      }
    }
    END {
      if (first == 0) {
        printf "\n  "
      }
    }' "${ENV_STATE_FILE}"),
  "environment_count": ${env_count},
  "event_count": ${event_count},
  "release_count": ${release_count}
}
EOF
}

cmd_history() {
  local limit="${DEFAULT_HISTORY_LIMIT}"
  local normalized_limit=''

  while (( $# > 0 )); do
    case "$1" in
      --limit)
        shift
        limit="${1:-}"
        ;;
      *)
        die "Unknown argument for history: $1" 2
        ;;
    esac
    shift
  done

  normalized_limit="$(normalize_limit "${limit}")"
  ensure_state_files

  cat <<EOF
{
  "status": "ok",
  "limit": ${normalized_limit},
  "events": $(tail -n "${normalized_limit}" "${EVENTS_FILE}" | tac | awk '
    BEGIN { first = 1; printf "[" }
    {
      if (length($0) == 0) {
        next
      }
      if (first == 1) {
        first = 0
      } else {
        printf ","
      }
      printf "\n    %s", $0
    }
    END {
      if (first == 0) {
        printf "\n  "
      }
      printf "]"
    }')
}
EOF
}

main() {
  local command="${1:-}"

  require_command awk
  require_command grep
  require_command sed
  require_command tail
  require_command tac
  require_command date

  case "${command}" in
    init)
      shift
      cmd_init "$@"
      ;;
    status)
      shift
      cmd_status "$@"
      ;;
    set-env)
      shift
      cmd_set_env "$@"
      ;;
    event)
      shift
      cmd_event "$@"
      ;;
    release)
      shift
      cmd_release "$@"
      ;;
    history)
      shift
      cmd_history "$@"
      ;;
    help|-h|--help|"")
      print_usage
      ;;
    *)
      print_usage >&2
      die "Unknown command: ${command}" 2
      ;;
  esac
}

main "$@"
