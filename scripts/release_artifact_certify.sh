#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Validate release artifact integrity against a provided SHA-256 sidecar.
#
# Operational Intent:
#   - Fail closed on missing artifacts or checksum files
#   - Perform deterministic integrity verification
#   - Emit operator-readable certification status
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

ARTIFACT_PATH="${1:-${ARTIFACT_PATH:-}}"
CHECKSUM_PATH="${2:-${CHECKSUM_PATH:-}}"

log_info() {
  printf '[release-certify][info] %s\n' "$*"
}

log_error() {
  printf '[release-certify][error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"

  log_error "${message}"
  exit "${exit_code}"
}

main() {
  command -v sha256sum >/dev/null 2>&1 || die "Missing required command: sha256sum" 2
  [[ -n "${ARTIFACT_PATH}" ]] || die "Artifact path must be provided as the first argument or ARTIFACT_PATH environment variable." 2
  [[ -n "${CHECKSUM_PATH}" ]] || die "Checksum path must be provided as the second argument or CHECKSUM_PATH environment variable." 2
  [[ -f "${ARTIFACT_PATH}" ]] || die "Artifact does not exist: ${ARTIFACT_PATH}" 2
  [[ -f "${CHECKSUM_PATH}" ]] || die "Checksum file does not exist: ${CHECKSUM_PATH}" 2

  local expected_checksum
  local actual_checksum

  expected_checksum="$(awk '{print $1}' "${CHECKSUM_PATH}")"
  actual_checksum="$(sha256sum "${ARTIFACT_PATH}" | awk '{print $1}')"

  [[ -n "${expected_checksum}" ]] || die "Checksum file is empty or malformed: ${CHECKSUM_PATH}" 2

  if [[ "${expected_checksum}" != "${actual_checksum}" ]]; then
    die "Artifact certification failed. Expected ${expected_checksum}, got ${actual_checksum}." 3
  fi

  log_info "Artifact certification passed."
  log_info "Artifact: ${ARTIFACT_PATH}"
  log_info "SHA-256: ${actual_checksum}"
}

main "$@"
