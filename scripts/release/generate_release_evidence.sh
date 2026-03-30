#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
#
# ------------------------------------------------------------------------------
# AOXC Release Evidence Generator
# ------------------------------------------------------------------------------
# Purpose:
#   Generate a deterministic, reviewable, and policy-enforced release evidence
#   package for the AOXC CLI binary.
#
# Operational intent:
#   - Execute release-critical validation gates prior to artifact approval.
#   - Produce integrity, audit, SBOM, provenance, and traceability evidence.
#   - Fail closed when mandatory evidence is required but unavailable.
#   - Avoid unsafe shell evaluation and nondeterministic temporary file usage.
#   - Emit a human-readable report and a machine-readable artifact index.
#
# Security posture:
#   - No use of 'eval'.
#   - No implicit shell parsing of externally supplied commands.
#   - All optional evidence generators must be provided as executable paths.
#   - Temporary files are isolated and automatically removed on exit.
#
# Usage examples:
#   ./scripts/release/generate_release_evidence.sh
#
#   ENFORCE_MANDATORY_EVIDENCE=0 \
#   AOXC_SIGNING_BIN=/usr/local/bin/aoxc-sign \
#   AOXC_PROVENANCE_BIN=/usr/local/bin/aoxc-provenance \
#   ./scripts/release/generate_release_evidence.sh
#
# Environment variables:
#   ARTIFACT_DIR                 Output directory for release evidence artifacts.
#   RELEASE_ID                   Stable release evidence identifier.
#   AOXC_BIN_PATH                Built AOXC binary path.
#   ENFORCE_MANDATORY_EVIDENCE   1 to fail closed if signature/provenance are missing.
#   AOXC_SIGNING_BIN             Optional executable used to generate signature evidence.
#   AOXC_PROVENANCE_BIN          Optional executable used to generate provenance evidence.

set -Eeuo pipefail
IFS=$'\n\t'

# ------------------------------------------------------------------------------
# Configuration
# ------------------------------------------------------------------------------
ARTIFACT_DIR="${ARTIFACT_DIR:-artifacts/release-evidence}"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
AOXC_BIN_PATH="${AOXC_BIN_PATH:-target/release/aoxc}"
ENFORCE_MANDATORY_EVIDENCE="${ENFORCE_MANDATORY_EVIDENCE:-1}"

# Optional generators must be executable paths, not shell fragments.
AOXC_SIGNING_BIN="${AOXC_SIGNING_BIN:-}"
AOXC_PROVENANCE_BIN="${AOXC_PROVENANCE_BIN:-}"

# ------------------------------------------------------------------------------
# Derived artifact paths
# ------------------------------------------------------------------------------
SHA256_FILE="${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sha256"
BUILD_MANIFEST_FILE="${ARTIFACT_DIR}/build-manifest-${RELEASE_ID}.json"
COMPAT_MATRIX_FILE="${ARTIFACT_DIR}/compat-matrix-${RELEASE_ID}.json"
PRODUCTION_AUDIT_FILE="${ARTIFACT_DIR}/production-audit-${RELEASE_ID}.json"
CARGO_AUDIT_FILE="${ARTIFACT_DIR}/cargo-audit-${RELEASE_ID}.json"
SBOM_FILE="${ARTIFACT_DIR}/sbom-${RELEASE_ID}.json"
SIGNATURE_FILE="${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sig"
SIGNATURE_STATUS_FILE="${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sig.status"
PROVENANCE_FILE="${ARTIFACT_DIR}/provenance-${RELEASE_ID}.json"
REPORT_FILE="${ARTIFACT_DIR}/release-evidence-${RELEASE_ID}.md"
ARTIFACT_INDEX_FILE="${ARTIFACT_DIR}/artifact-index-${RELEASE_ID}.json"
GIT_COMMIT_FILE="${ARTIFACT_DIR}/git-commit-${RELEASE_ID}.txt"
GIT_STATUS_FILE="${ARTIFACT_DIR}/git-status-${RELEASE_ID}.txt"
RUSTC_VERSION_FILE="${ARTIFACT_DIR}/rustc-version-${RELEASE_ID}.txt"
CARGO_VERSION_FILE="${ARTIFACT_DIR}/cargo-version-${RELEASE_ID}.txt"
FAILURE_REPORT_FILE="${ARTIFACT_DIR}/failure-${RELEASE_ID}.log"

# ------------------------------------------------------------------------------
# Temporary workspace
# ------------------------------------------------------------------------------
TMP_DIR="$(mktemp -d)"
METADATA_FILE="${TMP_DIR}/cargo-metadata.json"

# ------------------------------------------------------------------------------
# Runtime state
# ------------------------------------------------------------------------------
SIGNATURE_PRESENT=0
PROVENANCE_PRESENT=0
SBOM_GENERATOR="cargo metadata fallback"
CARGO_AUDIT_PRESENT=0
SCRIPT_START_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# ------------------------------------------------------------------------------
# Logging and failure handling
# ------------------------------------------------------------------------------
log() {
  printf '[info] %s\n' "$*"
}

warn() {
  printf '[warn] %s\n' "$*" >&2
}

fail() {
  printf '[error] %s\n' "$*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

require_file() {
  [[ -f "$1" ]] || fail "Missing required file: $1"
}

require_directory() {
  [[ -d "$1" ]] || fail "Missing required directory: $1"
}

cleanup() {
  rm -rf "${TMP_DIR}"
}

on_error() {
  local exit_code="$1"
  local failed_line="$2"

  mkdir -p "${ARTIFACT_DIR}"

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
sbom_generator=${SBOM_GENERATOR}
EOF

  printf '[error] Release evidence generation failed at line %s with exit code %s\n' "${failed_line}" "${exit_code}" >&2
}

trap 'on_error "$?" "$LINENO"' ERR
trap cleanup EXIT

# ------------------------------------------------------------------------------
# JSON writers
# ------------------------------------------------------------------------------
write_json_file() {
  local output_path="$1"
  shift
  cat > "${output_path}" <<EOF
$*
EOF
}

# ------------------------------------------------------------------------------
# Dependency validation
# ------------------------------------------------------------------------------
require_command cargo
require_command sha256sum
require_command python3
require_command mktemp
require_command git

mkdir -p "${ARTIFACT_DIR}"
require_directory "${ARTIFACT_DIR}"

# ------------------------------------------------------------------------------
# Workspace evidence capture
# ------------------------------------------------------------------------------
log "Capturing toolchain versions"
rustc --version > "${RUSTC_VERSION_FILE}"
cargo --version > "${CARGO_VERSION_FILE}"

log "Capturing Git commit reference"
git rev-parse HEAD > "${GIT_COMMIT_FILE}"

log "Capturing Git working tree status"
git status --short > "${GIT_STATUS_FILE}"

# ------------------------------------------------------------------------------
# Release-critical validation and build pipeline
# ------------------------------------------------------------------------------
log "Running formatting gate"
cargo fmt --all --check

log "Running release-critical test suite"
cargo test -p aoxcmd -- --test-threads=1

log "Building AOXC release binary"
cargo build --release -p aoxcmd --bin aoxc

require_file "${AOXC_BIN_PATH}"

# ------------------------------------------------------------------------------
# Integrity evidence
# ------------------------------------------------------------------------------
log "Generating release binary checksum"
sha256sum "${AOXC_BIN_PATH}" | tee "${SHA256_FILE}" >/dev/null

# ------------------------------------------------------------------------------
# AOXC native evidence
# ------------------------------------------------------------------------------
log "Generating AOXC build manifest"
"${AOXC_BIN_PATH}" build-manifest > "${BUILD_MANIFEST_FILE}"

log "Generating AOXC compatibility matrix"
"${AOXC_BIN_PATH}" compat-matrix > "${COMPAT_MATRIX_FILE}"

log "Generating AOXC production audit report"
"${AOXC_BIN_PATH}" production-audit --format json > "${PRODUCTION_AUDIT_FILE}"

# ------------------------------------------------------------------------------
# Cargo audit evidence
# ------------------------------------------------------------------------------
if cargo audit --help >/dev/null 2>&1; then
  log "Generating cargo-audit report"
  cargo audit --json > "${CARGO_AUDIT_FILE}"
  CARGO_AUDIT_PRESENT=1
else
  warn "cargo-audit is not available; generating explicit placeholder report"
  write_json_file "${CARGO_AUDIT_FILE}" \
'{
  "status": "missing-tool",
  "requirement": "Install cargo-audit to generate vulnerability evidence for release approval"
}'
fi

# ------------------------------------------------------------------------------
# SBOM generation
# ------------------------------------------------------------------------------
if command -v cargo-cyclonedx >/dev/null 2>&1; then
  log "Generating SBOM via cargo-cyclonedx"
  cargo cyclonedx --format json --output-file "${SBOM_FILE}"
  SBOM_GENERATOR="cargo-cyclonedx"
else
  log "cargo-cyclonedx not found; falling back to cargo metadata derived SBOM"
  cargo metadata --format-version 1 --locked > "${METADATA_FILE}"

  python3 - "${RELEASE_ID}" "${SBOM_FILE}" "${METADATA_FILE}" <<'PY'
import json
import sys
from pathlib import Path

release_id = sys.argv[1]
output_path = Path(sys.argv[2])
metadata_path = Path(sys.argv[3])

with metadata_path.open("r", encoding="utf-8") as handle:
    metadata = json.load(handle)

packages = []
for package in metadata.get("packages", []):
    dependencies = sorted(
        dep["name"] for dep in package.get("dependencies", [])
        if isinstance(dep, dict) and "name" in dep
    )
    packages.append(
        {
            "name": package.get("name"),
            "version": package.get("version"),
            "id": package.get("id"),
            "manifest_path": package.get("manifest_path"),
            "dependencies": dependencies,
        }
    )

document = {
    "bomFormat": "AOXC-SBOM",
    "specVersion": "1.0",
    "serialNumber": f"urn:aoxc:sbom:{release_id}",
    "metadata": {
        "generated_by": "scripts/release/generate_release_evidence.sh",
        "source": "cargo metadata --format-version 1 --locked",
    },
    "components": packages,
}

with output_path.open("w", encoding="utf-8") as handle:
    json.dump(document, handle, indent=2)
    handle.write("\n")
PY
fi

require_file "${SBOM_FILE}"

# ------------------------------------------------------------------------------
# Signature evidence
# ------------------------------------------------------------------------------
if [[ -n "${AOXC_SIGNING_BIN}" ]]; then
  require_command "${AOXC_SIGNING_BIN}"
  log "Generating signature evidence"
  "${AOXC_SIGNING_BIN}" "${AOXC_BIN_PATH}" > "${SIGNATURE_FILE}"
  require_file "${SIGNATURE_FILE}"
  SIGNATURE_PRESENT=1
else
  warn "Signature generator is not configured"
  printf 'MISSING_SIGNATURE\n' > "${SIGNATURE_STATUS_FILE}"
fi

# ------------------------------------------------------------------------------
# Provenance evidence
# ------------------------------------------------------------------------------
if [[ -n "${AOXC_PROVENANCE_BIN}" ]]; then
  require_command "${AOXC_PROVENANCE_BIN}"
  log "Generating provenance evidence"
  "${AOXC_PROVENANCE_BIN}" "${AOXC_BIN_PATH}" > "${PROVENANCE_FILE}"
  require_file "${PROVENANCE_FILE}"
  PROVENANCE_PRESENT=1
else
  warn "Provenance generator is not configured"
  write_json_file "${PROVENANCE_FILE}" \
'{
  "status": "missing-generator",
  "requirement": "Set AOXC_PROVENANCE_BIN to an executable that emits a provenance attestation before release approval"
}'
fi

# ------------------------------------------------------------------------------
# Enforced release policy
# ------------------------------------------------------------------------------
if [[ "${ENFORCE_MANDATORY_EVIDENCE}" == "1" ]]; then
  [[ "${SIGNATURE_PRESENT}" == "1" ]] || fail "Release evidence policy violation: signature artifact is missing"
  [[ "${PROVENANCE_PRESENT}" == "1" ]] || fail "Release evidence policy violation: provenance artifact is missing"
fi

# ------------------------------------------------------------------------------
# Artifact index
# ------------------------------------------------------------------------------
log "Generating machine-readable artifact index"
python3 - "${ARTIFACT_INDEX_FILE}" \
  "${RELEASE_ID}" \
  "${AOXC_BIN_PATH}" \
  "${SCRIPT_START_UTC}" \
  "${SHA256_FILE}" \
  "${BUILD_MANIFEST_FILE}" \
  "${COMPAT_MATRIX_FILE}" \
  "${PRODUCTION_AUDIT_FILE}" \
  "${CARGO_AUDIT_FILE}" \
  "${SBOM_FILE}" \
  "${SIGNATURE_FILE}" \
  "${SIGNATURE_STATUS_FILE}" \
  "${PROVENANCE_FILE}" \
  "${REPORT_FILE}" \
  "${GIT_COMMIT_FILE}" \
  "${GIT_STATUS_FILE}" \
  "${RUSTC_VERSION_FILE}" \
  "${CARGO_VERSION_FILE}" \
  "${SIGNATURE_PRESENT}" \
  "${PROVENANCE_PRESENT}" \
  "${CARGO_AUDIT_PRESENT}" \
  "${SBOM_GENERATOR}" \
<<'PY'
import json
import sys
from pathlib import Path

(
    output_path,
    release_id,
    binary_path,
    generated_at,
    sha256_file,
    build_manifest_file,
    compat_matrix_file,
    production_audit_file,
    cargo_audit_file,
    sbom_file,
    signature_file,
    signature_status_file,
    provenance_file,
    report_file,
    git_commit_file,
    git_status_file,
    rustc_version_file,
    cargo_version_file,
    signature_present,
    provenance_present,
    cargo_audit_present,
    sbom_generator,
) = sys.argv[1:]

def present(path_str: str) -> bool:
    return Path(path_str).is_file()

document = {
    "release_id": release_id,
    "generated_at_utc": generated_at,
    "binary_path": binary_path,
    "policy": {
        "enforce_mandatory_evidence": True,
        "signature_present": signature_present == "1",
        "provenance_present": provenance_present == "1",
        "cargo_audit_present": cargo_audit_present == "1",
        "sbom_generator": sbom_generator,
    },
    "artifacts": {
        "checksum": {"path": sha256_file, "present": present(sha256_file)},
        "build_manifest": {"path": build_manifest_file, "present": present(build_manifest_file)},
        "compat_matrix": {"path": compat_matrix_file, "present": present(compat_matrix_file)},
        "production_audit": {"path": production_audit_file, "present": present(production_audit_file)},
        "cargo_audit": {"path": cargo_audit_file, "present": present(cargo_audit_file)},
        "sbom": {"path": sbom_file, "present": present(sbom_file)},
        "signature": {"path": signature_file, "present": present(signature_file)},
        "signature_status": {"path": signature_status_file, "present": present(signature_status_file)},
        "provenance": {"path": provenance_file, "present": present(provenance_file)},
        "report": {"path": report_file, "present": present(report_file)},
        "git_commit": {"path": git_commit_file, "present": present(git_commit_file)},
        "git_status": {"path": git_status_file, "present": present(git_status_file)},
        "rustc_version": {"path": rustc_version_file, "present": present(rustc_version_file)},
        "cargo_version": {"path": cargo_version_file, "present": present(cargo_version_file)},
    },
}

with open(output_path, "w", encoding="utf-8") as handle:
    json.dump(document, handle, indent=2)
    handle.write("\n")
PY

require_file "${ARTIFACT_INDEX_FILE}"

# ------------------------------------------------------------------------------
# Human-readable release evidence report
# ------------------------------------------------------------------------------
log "Generating human-readable release evidence report"
cat > "${REPORT_FILE}" <<EOF
# AOXC Release Evidence ${RELEASE_ID}

- release_id: ${RELEASE_ID}
- generated_at_utc: ${SCRIPT_START_UTC}
- binary_path: ${AOXC_BIN_PATH}

## Validation Pipeline
- cargo fmt --all --check
- cargo test -p aoxcmd -- --test-threads=1
- cargo build --release -p aoxcmd --bin aoxc

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

log "Release evidence generated successfully under ${ARTIFACT_DIR}"
