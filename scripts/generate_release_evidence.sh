#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Generate deterministic release evidence for AOXC release artifacts.
#
# Operational Intent:
#   - Validate that the target release artifact exists
#   - Produce a checksum sidecar
#   - Emit a compact machine-readable evidence manifest
#   - Fail closed on missing prerequisites or invalid input
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

AOXC_ROOT="${AOXC_ROOT:-${HOME}/.aoxc}"
RELEASES_DIR="${RELEASES_DIR:-${AOXC_ROOT}/releases}"
ARTIFACT_PATH="${1:-${ARTIFACT_PATH:-}}"
EVIDENCE_DIR="${EVIDENCE_DIR:-${RELEASES_DIR}/evidence}"

log_info() {
  printf '[release-evidence][info] %s\n' "$*"
}

log_error() {
  printf '[release-evidence][error] %s\n' "$*" >&2
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
  [[ -f "${ARTIFACT_PATH}" ]] || die "Artifact does not exist: ${ARTIFACT_PATH}" 2

  mkdir -p "${EVIDENCE_DIR}"

  local artifact_name
  local checksum_file
  local manifest_file
  local checksum_value

  artifact_name="$(basename "${ARTIFACT_PATH}")"
  checksum_file="${EVIDENCE_DIR}/${artifact_name}.sha256"
  manifest_file="${EVIDENCE_DIR}/${artifact_name}.evidence.json"
  checksum_value="$(sha256sum "${ARTIFACT_PATH}" | awk '{print $1}')"

  printf '%s  %s\n' "${checksum_value}" "${artifact_name}" > "${checksum_file}"

  cat > "${manifest_file}" <<MANIFEST
{
  "artifact_name": "${artifact_name}",
  "artifact_path": "${ARTIFACT_PATH}",
  "sha256": "${checksum_value}",
  "generated_at_utc": "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)",
  "generator": "scripts/release/generate_release_evidence.sh"
}
MANIFEST

  log_info "Release evidence generated successfully."
  log_info "Checksum file: ${checksum_file}"
  log_info "Manifest file: ${manifest_file}"
}

main "$@"
