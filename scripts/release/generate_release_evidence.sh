#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Generate a deterministic, reviewable, and policy-enforced release evidence
#   package for the AOXC CLI binary.
#
# Operational Intent:
#   - Execute release-critical validation gates prior to artifact approval
#   - Produce integrity, audit, SBOM, provenance, and traceability evidence
#   - Fail closed when mandatory evidence is required but unavailable
#   - Avoid unsafe shell evaluation and nondeterministic temporary file usage
#   - Emit a human-readable report and a machine-readable artifact index
#
# Security Posture:
#   - No use of eval
#   - No implicit shell parsing of externally supplied commands
#   - Optional evidence generators must be provided as executable paths
#   - Temporary files are isolated and automatically removed on exit
#
# Usage:
#   ./scripts/release/generate_release_evidence.sh
#
#   ENFORCE_MANDATORY_EVIDENCE=0 \
#   AOXC_SIGNING_BIN=/usr/local/bin/aoxc-sign \
#   AOXC_PROVENANCE_BIN=/usr/local/bin/aoxc-provenance \
#   ./scripts/release/generate_release_evidence.sh
#
# Environment Variables:
#   ARTIFACT_DIR                 Output directory for release evidence artifacts
#   RELEASE_ID                   Stable release evidence identifier
#   AOXC_BIN_PATH                Built AOXC binary path
#   ENFORCE_MANDATORY_EVIDENCE   1 to fail closed if signature/provenance are missing
#   AOXC_SIGNING_BIN             Optional executable used to generate signature evidence
#   AOXC_PROVENANCE_BIN          Optional executable used to generate provenance evidence
#
# Exit Codes:
#   0  Successful completion
#   1  General operational failure
#   2  Invalid configuration or missing dependency
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly DEFAULT_ARTIFACT_DIR="artifacts/release-evidence"
readonly DEFAULT_AOXC_BIN_PATH="target/release/aoxc"
readonly DEFAULT_ENFORCE_MANDATORY_EVIDENCE="1"

ARTIFACT_DIR="${ARTIFACT_DIR:-$DEFAULT_ARTIFACT_DIR}"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
AOXC_BIN_PATH="${AOXC_BIN_PATH:-$DEFAULT_AOXC_BIN_PATH}"
ENFORCE_MANDATORY_EVIDENCE="${ENFORCE_MANDATORY_EVIDENCE:-$DEFAULT_ENFORCE_MANDATORY_EVIDENCE}"

AOXC_SIGNING_BIN="${AOXC_SIGNING_BIN:-}"
AOXC_PROVENANCE_BIN="${AOXC_PROVENANCE_BIN:-}"

readonly SHA256_FILE="${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sha256"
readonly BUILD_MANIFEST_FILE="${ARTIFACT_DIR}/build-manifest-${RELEASE_ID}.json"
readonly COMPAT_MATRIX_FILE="${ARTIFACT_DIR}/compat-matrix-${RELEASE_ID}.json"
readonly PRODUCTION_AUDIT_FILE="${ARTIFACT_DIR}/production-audit-${RELEASE_ID}.json"
readonly CARGO_AUDIT_FILE="${ARTIFACT_DIR}/cargo-audit-${RELEASE_ID}.json"
readonly SBOM_FILE="${ARTIFACT_DIR}/sbom-${RELEASE_ID}.json"
readonly SIGNATURE_FILE="${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sig"
readonly SIGNATURE_STATUS_FILE="${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sig.status"
readonly PROVENANCE_FILE="${ARTIFACT_DIR}/provenance-${RELEASE_ID}.json"
readonly REPORT_FILE="${ARTIFACT_DIR}/release-evidence-${RELEASE_ID}.md"
readonly ARTIFACT_INDEX_FILE="${ARTIFACT_DIR}/artifact-index-${RELEASE_ID}.json"
readonly GIT_COMMIT_FILE="${ARTIFACT_DIR}/git-commit-${RELEASE_ID}.txt"
readonly GIT_STATUS_FILE="${ARTIFACT_DIR}/git-status-${RELEASE_ID}.txt"
readonly RUSTC_VERSION_FILE="${ARTIFACT_DIR}/rustc-version-${RELEASE_ID}.txt"
readonly CARGO_VERSION_FILE="${ARTIFACT_DIR}/cargo-version-${RELEASE_ID}.txt"
readonly FAILURE_REPORT_FILE="${ARTIFACT_DIR}/failure-${RELEASE_ID}.log"

TMP_DIR=''
METADATA_FILE=''

SIGNATURE_PRESENT=0
PROVENANCE_PRESENT=0
CARGO_AUDIT_PRESENT=0
SBOM_GENERATOR='cargo metadata shell fallback'
SCRIPT_START_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

log_info() {
  printf '[info] %s\n' "$*"
}

log_warn() {
  printf '[warn] %s\n' "$*" >&2
}

log_error() {
  printf '[error] %s\n' "$*" >&2
}

fail() {
  local message="$1"
  log_error "${message}"
  exit 1
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || fail "Missing required command: ${command_name}"
}

require_file() {
  local file_path="$1"
  [[ -f "${file_path}" ]] || fail "Missing required file: ${file_path}"
}

ensure_directory() {
  local dir_path="$1"

  if [[ -e "${dir_path}" && ! -d "${dir_path}" ]]; then
    fail "Path exists but is not a directory: ${dir_path}"
  fi

  mkdir -p "${dir_path}"
}

validate_configuration() {
  case "${ENFORCE_MANDATORY_EVIDENCE}" in
    0|1) ;;
    *)
      fail "ENFORCE_MANDATORY_EVIDENCE must be 0 or 1"
      ;;
  esac
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

json_bool() {
  local value="$1"
  if [[ "${value}" == "1" ]]; then
    printf 'true'
  else
    printf 'false'
  fi
}

cleanup() {
  if [[ -n "${TMP_DIR}" && -d "${TMP_DIR}" ]]; then
    rm -rf "${TMP_DIR}"
  fi
}

on_error() {
  local exit_code="$1"
  local failed_line="$2"

  ensure_directory "${ARTIFACT_DIR}"

  cat > "${FAILURE_REPORT_FILE}" <<EOF
timestamp_utc=${SCRIPT_START_UTC}
release_id=${RELEASE_ID}
status=failed
exit_code=${exit_code}
failed_line=${failed_line}
artifact_dir=${ARTIFACT_DIR}
binary_path=${AOXC_BIN_PATH}
signature_present=${SIGNATURE_PRESENT}
provenance_present=${PROVENANCE_PRESENT}
cargo_audit_present=${CARGO_AUDIT_PRESENT}
sbom_generator=${SBOM_GENERATOR}
EOF

  log_error "Release evidence generation failed at line ${failed_line} with exit code ${exit_code}"
}

trap 'on_error "$?" "$LINENO"' ERR
trap cleanup EXIT

write_json_file() {
  local output_path="$1"
  shift
  cat > "${output_path}" <<EOF
$*
EOF
}

capture_toolchain_versions() {
  log_info "Capturing toolchain versions"
  rustc --version > "${RUSTC_VERSION_FILE}"
  cargo --version > "${CARGO_VERSION_FILE}"
}

capture_git_state() {
  log_info "Capturing Git commit reference"
  git rev-parse HEAD > "${GIT_COMMIT_FILE}"

  log_info "Capturing Git working tree status"
  git status --short > "${GIT_STATUS_FILE}"
}

run_release_validation_pipeline() {
  log_info "Running formatting gate"
  cargo fmt --all --check

  log_info "Running release-critical test suite"
  cargo test --locked --workspace --exclude aoxchub --all-targets --no-fail-fast

  log_info "Building AOXC release binary"
  cargo build --locked --release -p aoxcmd --bin aoxc

  require_file "${AOXC_BIN_PATH}"
}

generate_integrity_evidence() {
  log_info "Generating release binary checksum"
  sha256sum "${AOXC_BIN_PATH}" | tee "${SHA256_FILE}" >/dev/null
}

generate_native_aoxc_evidence() {
  log_info "Generating AOXC build manifest"
  "${AOXC_BIN_PATH}" build-manifest > "${BUILD_MANIFEST_FILE}"

  log_info "Generating AOXC compatibility matrix"
  "${AOXC_BIN_PATH}" compat-matrix > "${COMPAT_MATRIX_FILE}"

  log_info "Generating AOXC production audit report"
  "${AOXC_BIN_PATH}" production-audit --format json > "${PRODUCTION_AUDIT_FILE}"
}

generate_cargo_audit_evidence() {
  if command -v cargo-audit >/dev/null 2>&1; then
    log_info "Generating cargo-audit report"
    cargo-audit --json > "${CARGO_AUDIT_FILE}"
    CARGO_AUDIT_PRESENT=1
    return 0
  fi

  log_warn "cargo-audit is not available; generating explicit placeholder report"
  write_json_file "${CARGO_AUDIT_FILE}" \
'{
  "status": "missing-tool",
  "requirement": "Install cargo-audit to generate vulnerability evidence for release approval"
}'
}

generate_sbom_with_cargo_cyclonedx() {
  log_info "Generating SBOM via cargo-cyclonedx"
  cargo cyclonedx --format json --output-file "${SBOM_FILE}"
  SBOM_GENERATOR='cargo-cyclonedx'
}

generate_sbom_with_metadata_fallback() {
  local package_lines=''
  local package_name=''
  local package_version=''
  local package_manifest=''

  log_info "cargo-cyclonedx not found; falling back to cargo metadata derived SBOM"
  cargo metadata --format-version 1 --locked > "${METADATA_FILE}"

  package_lines="$(
    awk '
      BEGIN {
        in_packages = 0
        pkg_name = ""
        pkg_version = ""
        pkg_manifest = ""
      }
      /"packages":[[]/ { in_packages = 1 }
      in_packages && /"name":"[^"]+"/ && pkg_name == "" {
        match($0, /"name":"[^"]+"/)
        pkg_name = substr($0, RSTART + 8, RLENGTH - 9)
      }
      in_packages && /"version":"[^"]+"/ && pkg_version == "" {
        match($0, /"version":"[^"]+"/)
        pkg_version = substr($0, RSTART + 11, RLENGTH - 12)
      }
      in_packages && /"manifest_path":"[^"]+"/ && pkg_manifest == "" {
        match($0, /"manifest_path":"[^"]+"/)
        pkg_manifest = substr($0, RSTART + 17, RLENGTH - 18)
      }
      in_packages && pkg_name != "" && pkg_version != "" && pkg_manifest != "" {
        printf "%s\t%s\t%s\n", pkg_name, pkg_version, pkg_manifest
        pkg_name = ""
        pkg_version = ""
        pkg_manifest = ""
      }
      in_packages && /^[[:space:]]*][[:space:]]*[,}]?[[:space:]]*$/ { in_packages = 0 }
    ' "${METADATA_FILE}"
  )"

  {
    printf '{\n'
    printf '  "bomFormat": "AOXC-SBOM",\n'
    printf '  "specVersion": "1.0",\n'
    printf '  "serialNumber": "urn:aoxc:sbom:%s",\n' "$(escape_json_string "${RELEASE_ID}")"
    printf '  "metadata": {\n'
    printf '    "generated_by": "scripts/release/generate_release_evidence.sh",\n'
    printf '    "source": "cargo metadata --format-version 1 --locked"\n'
    printf '  },\n'
    printf '  "components": [\n'

    local first=1
    while IFS=$'\t' read -r package_name package_version package_manifest; do
      [[ -n "${package_name}" ]] || continue

      if (( first == 0 )); then
        printf ',\n'
      fi
      first=0

      printf '    {\n'
      printf '      "name": "%s",\n' "$(escape_json_string "${package_name}")"
      printf '      "version": "%s",\n' "$(escape_json_string "${package_version}")"
      printf '      "manifest_path": "%s"\n' "$(escape_json_string "${package_manifest}")"
      printf '    }'
    done <<< "${package_lines}"

    if (( first == 0 )); then
      printf '\n'
    fi

    printf '  ]\n'
    printf '}\n'
  } > "${SBOM_FILE}"
}

generate_sbom_evidence() {
  if command -v cargo-cyclonedx >/dev/null 2>&1; then
    generate_sbom_with_cargo_cyclonedx
  else
    generate_sbom_with_metadata_fallback
  fi

  require_file "${SBOM_FILE}"
}

generate_signature_evidence() {
  if [[ -n "${AOXC_SIGNING_BIN}" ]]; then
    [[ -x "${AOXC_SIGNING_BIN}" ]] || fail "AOXC_SIGNING_BIN is not executable: ${AOXC_SIGNING_BIN}"
    log_info "Generating signature evidence"
    "${AOXC_SIGNING_BIN}" "${AOXC_BIN_PATH}" > "${SIGNATURE_FILE}"
    require_file "${SIGNATURE_FILE}"
    SIGNATURE_PRESENT=1
    return 0
  fi

  log_warn "Signature generator is not configured"
  printf 'MISSING_SIGNATURE\n' > "${SIGNATURE_STATUS_FILE}"
}

generate_provenance_evidence() {
  if [[ -n "${AOXC_PROVENANCE_BIN}" ]]; then
    [[ -x "${AOXC_PROVENANCE_BIN}" ]] || fail "AOXC_PROVENANCE_BIN is not executable: ${AOXC_PROVENANCE_BIN}"
    log_info "Generating provenance evidence"
    "${AOXC_PROVENANCE_BIN}" "${AOXC_BIN_PATH}" > "${PROVENANCE_FILE}"
    require_file "${PROVENANCE_FILE}"
    PROVENANCE_PRESENT=1
    return 0
  fi

  log_warn "Provenance generator is not configured"
  write_json_file "${PROVENANCE_FILE}" \
'{
  "status": "missing-generator",
  "requirement": "Set AOXC_PROVENANCE_BIN to an executable that emits a provenance attestation before release approval"
}'
}

enforce_release_policy() {
  if [[ "${ENFORCE_MANDATORY_EVIDENCE}" == "1" ]]; then
    [[ "${SIGNATURE_PRESENT}" == "1" ]] || fail "Release evidence policy violation: signature artifact is missing"
    [[ "${PROVENANCE_PRESENT}" == "1" ]] || fail "Release evidence policy violation: provenance artifact is missing"
  fi
}

generate_artifact_index() {
  log_info "Generating machine-readable artifact index"

  cat > "${ARTIFACT_INDEX_FILE}" <<EOF
{
  "release_id": "$(escape_json_string "${RELEASE_ID}")",
  "generated_at_utc": "$(escape_json_string "${SCRIPT_START_UTC}")",
  "binary_path": "$(escape_json_string "${AOXC_BIN_PATH}")",
  "policy": {
    "enforce_mandatory_evidence": ${ENFORCE_MANDATORY_EVIDENCE},
    "signature_present": $(json_bool "${SIGNATURE_PRESENT}"),
    "provenance_present": $(json_bool "${PROVENANCE_PRESENT}"),
    "cargo_audit_present": $(json_bool "${CARGO_AUDIT_PRESENT}"),
    "sbom_generator": "$(escape_json_string "${SBOM_GENERATOR}")"
  },
  "artifacts": {
    "checksum": { "path": "$(escape_json_string "${SHA256_FILE}")", "present": $( [[ -f "${SHA256_FILE}" ]] && printf 'true' || printf 'false' ) },
    "build_manifest": { "path": "$(escape_json_string "${BUILD_MANIFEST_FILE}")", "present": $( [[ -f "${BUILD_MANIFEST_FILE}" ]] && printf 'true' || printf 'false' ) },
    "compat_matrix": { "path": "$(escape_json_string "${COMPAT_MATRIX_FILE}")", "present": $( [[ -f "${COMPAT_MATRIX_FILE}" ]] && printf 'true' || printf 'false' ) },
    "production_audit": { "path": "$(escape_json_string "${PRODUCTION_AUDIT_FILE}")", "present": $( [[ -f "${PRODUCTION_AUDIT_FILE}" ]] && printf 'true' || printf 'false' ) },
    "cargo_audit": { "path": "$(escape_json_string "${CARGO_AUDIT_FILE}")", "present": $( [[ -f "${CARGO_AUDIT_FILE}" ]] && printf 'true' || printf 'false' ) },
    "sbom": { "path": "$(escape_json_string "${SBOM_FILE}")", "present": $( [[ -f "${SBOM_FILE}" ]] && printf 'true' || printf 'false' ) },
    "signature": { "path": "$(escape_json_string "${SIGNATURE_FILE}")", "present": $( [[ -f "${SIGNATURE_FILE}" ]] && printf 'true' || printf 'false' ) },
    "signature_status": { "path": "$(escape_json_string "${SIGNATURE_STATUS_FILE}")", "present": $( [[ -f "${SIGNATURE_STATUS_FILE}" ]] && printf 'true' || printf 'false' ) },
    "provenance": { "path": "$(escape_json_string "${PROVENANCE_FILE}")", "present": $( [[ -f "${PROVENANCE_FILE}" ]] && printf 'true' || printf 'false' ) },
    "report": { "path": "$(escape_json_string "${REPORT_FILE}")", "present": $( [[ -f "${REPORT_FILE}" ]] && printf 'true' || printf 'false' ) },
    "git_commit": { "path": "$(escape_json_string "${GIT_COMMIT_FILE}")", "present": $( [[ -f "${GIT_COMMIT_FILE}" ]] && printf 'true' || printf 'false' ) },
    "git_status": { "path": "$(escape_json_string "${GIT_STATUS_FILE}")", "present": $( [[ -f "${GIT_STATUS_FILE}" ]] && printf 'true' || printf 'false' ) },
    "rustc_version": { "path": "$(escape_json_string "${RUSTC_VERSION_FILE}")", "present": $( [[ -f "${RUSTC_VERSION_FILE}" ]] && printf 'true' || printf 'false' ) },
    "cargo_version": { "path": "$(escape_json_string "${CARGO_VERSION_FILE}")", "present": $( [[ -f "${CARGO_VERSION_FILE}" ]] && printf 'true' || printf 'false' ) }
  }
}
EOF

  require_file "${ARTIFACT_INDEX_FILE}"
}

generate_human_readable_report() {
  log_info "Generating human-readable release evidence report"

  cat > "${REPORT_FILE}" <<EOF
# AOXC Release Evidence ${RELEASE_ID}

- release_id: ${RELEASE_ID}
- generated_at_utc: ${SCRIPT_START_UTC}
- binary_path: ${AOXC_BIN_PATH}

## Validation Pipeline
- cargo fmt --all --check
- cargo test --locked --workspace --exclude aoxchub --all-targets --no-fail-fast
- cargo build --locked --release -p aoxcmd --bin aoxc

## Release Evidence Artifacts
- artifact_checksum: $(basename "${SHA256_FILE}")
- artifact_build_manifest: $(basename "${BUILD_MANIFEST_FILE}")
- artifact_compatibility_matrix: $(basename "${COMPAT_MATRIX_FILE}")
- artifact_production_audit: $(basename "${PRODUCTION_AUDIT_FILE}")
- artifact_cargo_audit: $(basename "${CARGO_AUDIT_FILE}")
- artifact_sbom: $(basename "${SBOM_FILE}")
- artifact_signature: $(basename "${SIGNATURE_FILE}") or $(basename "${SIGNATURE_STATUS_FILE}")
- artifact_provenance: $(basename "${PROVENANCE_FILE}")
- artifact_git_commit: $(basename "${GIT_COMMIT_FILE}")
- artifact_git_status: $(basename "${GIT_STATUS_FILE}")
- artifact_rustc_version: $(basename "${RUSTC_VERSION_FILE}")
- artifact_cargo_version: $(basename "${CARGO_VERSION_FILE}")
- artifact_index: $(basename "${ARTIFACT_INDEX_FILE}")

## Policy Status
- enforcement_mode: ${ENFORCE_MANDATORY_EVIDENCE}
- signature_present: ${SIGNATURE_PRESENT}
- provenance_present: ${PROVENANCE_PRESENT}
- cargo_audit_present: ${CARGO_AUDIT_PRESENT}
- sbom_generator: ${SBOM_GENERATOR}

## Approval Rule
- release approval must be denied if mandatory evidence remains absent under enforced mode
EOF

  require_file "${REPORT_FILE}"
}

main() {
  validate_configuration

  require_command cargo
  require_command rustc
  require_command sha256sum
  require_command mktemp
  require_command git
  require_command awk
  require_command tee
  require_command basename
  require_command date

  ensure_directory "${ARTIFACT_DIR}"

  TMP_DIR="$(mktemp -d)"
  METADATA_FILE="${TMP_DIR}/cargo-metadata.json"

  capture_toolchain_versions
  capture_git_state
  run_release_validation_pipeline
  generate_integrity_evidence
  generate_native_aoxc_evidence
  generate_cargo_audit_evidence
  generate_sbom_evidence
  generate_signature_evidence
  generate_provenance_evidence
  enforce_release_policy
  generate_artifact_index
  generate_human_readable_report

  log_info "Release evidence generated successfully under ${ARTIFACT_DIR}"
}

main "$@"
