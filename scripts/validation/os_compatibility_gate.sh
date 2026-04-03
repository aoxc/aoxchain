#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARTIFACT_DIR="${AOXC_OS_COMPAT_ARTIFACT_DIR:-${REPO_ROOT}/artifacts/os-compat}"
STRICT="${AOXC_OS_COMPAT_STRICT:-0}"

missing=0
warnings=0

log() {
  echo "[os-compat-gate] $*"
}

require_file() {
  local f="$1"
  if [[ ! -f "${REPO_ROOT}/${f}" ]]; then
    log "missing required file: ${f}"
    missing=1
  fi
}

require_pattern() {
  local f="$1"
  local p="$2"
  if ! grep -Fq "${p}" "${REPO_ROOT}/${f}"; then
    log "missing expected pattern '${p}' in ${f}"
    missing=1
  fi
}

warn_if_missing_cmd() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    log "warning: command not available in current host: ${cmd}"
    warnings=1
  fi
}

# required cross-platform surfaces
require_file "Dockerfile"
require_file "docker-compose.yaml"
require_file "Makefile"
require_file "README.md"
require_file "READ.md"
require_file "docs/OS_COMPATIBILITY.md"

# ensure makefile keeps major platform paths explicit
require_pattern "Makefile" 'ifeq ($(OS),Windows_NT)'
require_pattern "Makefile" 'else ifeq ($(UNAME_S),Darwin)'
require_pattern "Makefile" "AOXC_PLATFORM := linux"

# ensure docs explicitly include requested OS surfaces
require_pattern "docs/OS_COMPATIBILITY.md" "NixOS"
require_pattern "docs/OS_COMPATIBILITY.md" "macOS"
require_pattern "docs/OS_COMPATIBILITY.md" "Windows"
require_pattern "docs/OS_COMPATIBILITY.md" "Docker"

# informational host availability probes (warning-only)
warn_if_missing_cmd bash
warn_if_missing_cmd make
warn_if_missing_cmd cargo
warn_if_missing_cmd git
warn_if_missing_cmd docker

mkdir -p "${ARTIFACT_DIR}"
SUMMARY_FILE="${ARTIFACT_DIR}/summary.json"
cat > "${SUMMARY_FILE}" <<JSON
{
  "suite": "os-compat-gate",
  "strict_mode": ${STRICT},
  "timestamp_utc": "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)",
  "host": {
    "uname": "$(uname -s 2>/dev/null || echo unknown)"
  },
  "status": {
    "missing_required_surfaces": ${missing},
    "warnings_present": ${warnings}
  }
}
JSON

if [[ "${missing}" -ne 0 ]]; then
  log "FAILED: required OS compatibility surfaces are missing"
  exit 1
fi

if [[ "${warnings}" -ne 0 && "${STRICT}" == "1" ]]; then
  log "FAILED (strict mode): host compatibility warnings present"
  exit 1
fi

if [[ "${warnings}" -ne 0 ]]; then
  log "WARNING: host lacks one or more optional tools for complete compatibility rehearsal"
fi

log "PASS: OS compatibility surfaces are present"
log "artifact: ${SUMMARY_FILE}"
