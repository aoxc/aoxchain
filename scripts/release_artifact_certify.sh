#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Generate deterministic release artifact integrity evidence and, when
#   configured, produce a detached cryptographic signature for the generated
#   manifest.
#
# Scope:
#   - Validate input artifacts
#   - Produce a SHA-256 checksum ledger
#   - Produce a structured JSON manifest for audit and release review
#   - Optionally sign the manifest with an operator-provided PEM key
#   - Optionally copy the corresponding signing certificate alongside outputs
#
# Usage:
#   ./scripts/release_artifact_certify.sh target/release/aoxc target/release/aoxckit
#
#   AOXC_RELEASE_SIGNING_KEY_PEM=key.pem \
#   AOXC_RELEASE_SIGNING_CERT_PEM=cert.pem \
#   ./scripts/release_artifact_certify.sh target/release/aoxc
#
# Exit Codes:
#   0  Successful completion
#   2  Invalid invocation
#   3  Missing or invalid artifact input
#   4  Incomplete signing configuration
#   5  Signing material path validation failure
#   6  Required output directory could not be prepared
#   7  Manifest signing failure
#   8  Manifest generation failure
#   9  Missing required host dependency
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_NAME="$(basename "$0")"
readonly DEFAULT_OUTPUT_DIR="./dist/release-artifacts"

OUT_DIR="${AOXC_RELEASE_ARTIFACT_DIR:-$DEFAULT_OUTPUT_DIR}"
MANIFEST_PATH="${OUT_DIR}/artifact-manifest.json"
CHECKSUMS_PATH="${OUT_DIR}/SHA256SUMS"
SIGNATURE_PATH="${OUT_DIR}/artifact-manifest.sig"
CERT_COPY_PATH="${OUT_DIR}/artifact-signing-cert.pem"

log_info() {
  printf '[info] %s\n' "$*"
}

log_warn() {
  printf '[warn] %s\n' "$*" >&2
}

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
  ${SCRIPT_NAME} <artifact> [<artifact> ...]

Environment:
  AOXC_RELEASE_ARTIFACT_DIR       Optional output directory
  AOXC_RELEASE_SIGNING_KEY_PEM    Optional PEM private key used for manifest signing
  AOXC_RELEASE_SIGNING_CERT_PEM   Optional PEM certificate copied with signed outputs
EOF
}

require_command() {
  local command_name="$1"

  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 9
}

require_file() {
  local file_path="$1"
  local description="$2"

  [[ -f "${file_path}" ]] || die "${description} does not exist: ${file_path}" 3
}

ensure_output_directory() {
  if [[ -e "${OUT_DIR}" && ! -d "${OUT_DIR}" ]]; then
    die "Configured output path exists but is not a directory: ${OUT_DIR}" 6
  fi

  mkdir -p "${OUT_DIR}" || die "Unable to create output directory: ${OUT_DIR}" 6
}

write_checksums() {
  local artifact_path=''

  : > "${CHECKSUMS_PATH}"

  for artifact_path in "$@"; do
    local digest=''
    local file_name=''

    digest="$(sha256sum "${artifact_path}" | awk '{print $1}')"
    file_name="$(basename "${artifact_path}")"

    printf '%s  %s\n' "${digest}" "${file_name}" >> "${CHECKSUMS_PATH}"
  done
}

write_manifest() {
  python3 - "$@" "${MANIFEST_PATH}" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

artifact_paths = sys.argv[1:-1]
manifest_path = sys.argv[-1]

artifacts = []
for raw_path in artifact_paths:
    path = Path(raw_path)
    stat_result = path.stat()
    artifacts.append(
        {
            "path": raw_path,
            "file_name": path.name,
            "sha256": __import__("hashlib").sha256(path.read_bytes()).hexdigest(),
            "size_bytes": stat_result.st_size,
        }
    )

manifest = {
    "schema_version": 1,
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "artifact_count": len(artifacts),
    "artifacts": artifacts,
}

with open(manifest_path, "w", encoding="utf-8") as handle:
    json.dump(manifest, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
}

validate_artifacts() {
  local artifact_path=''

  (( $# > 0 )) || {
    print_usage >&2
    die "At least one artifact path must be provided." 2
  }

  for artifact_path in "$@"; do
    if [[ ! -f "${artifact_path}" ]]; then
      die "Artifact does not exist: ${artifact_path}" 3
    fi

    if [[ ! -r "${artifact_path}" ]]; then
      die "Artifact is not readable: ${artifact_path}" 3
    fi
  done
}

handle_optional_signing() {
  local signing_key="${AOXC_RELEASE_SIGNING_KEY_PEM:-}"
  local signing_cert="${AOXC_RELEASE_SIGNING_CERT_PEM:-}"

  if [[ -z "${signing_key}" && -z "${signing_cert}" ]]; then
    log_info "Manifest signing was not requested."
    return 0
  fi

  require_command openssl

  if [[ -z "${signing_key}" || -z "${signing_cert}" ]]; then
    die "Both AOXC_RELEASE_SIGNING_KEY_PEM and AOXC_RELEASE_SIGNING_CERT_PEM are required when signing is enabled." 4
  fi

  [[ -f "${signing_key}" ]] || die "Signing key path does not exist: ${signing_key}" 5
  [[ -f "${signing_cert}" ]] || die "Signing certificate path does not exist: ${signing_cert}" 5

  log_info "Signing manifest with operator-provided PEM key."
  openssl dgst -sha256 -sign "${signing_key}" -out "${SIGNATURE_PATH}" "${MANIFEST_PATH}" \
    || die "Manifest signing failed." 7

  cp "${signing_cert}" "${CERT_COPY_PATH}" || die "Unable to copy signing certificate into output directory." 5
}

main() {
  require_command sha256sum
  require_command python3
  require_command awk
  require_command stat

  validate_artifacts "$@"
  ensure_output_directory

  log_info "Generating checksum ledger."
  write_checksums "$@"

  log_info "Generating JSON manifest."
  write_manifest "$@" || die "Manifest generation failed." 8

  handle_optional_signing

  printf '[ok] wrote %s\n' "${MANIFEST_PATH}"
  printf '[ok] wrote %s\n' "${CHECKSUMS_PATH}"

  if [[ -f "${SIGNATURE_PATH}" ]]; then
    printf '[ok] wrote %s\n' "${SIGNATURE_PATH}"
    printf '[ok] copied %s\n' "${CERT_COPY_PATH}"
  fi
}

main "$@"
