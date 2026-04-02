#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"
ARTIFACT_DIR="${AOXCVM_ARTIFACT_DIR:-${REPO_ROOT}/artifacts/aoxcvm-phase3}"
BUNDLE_DIR="${ARTIFACT_DIR}/evidence-bundle"
MATRIX_FILE="${ARTIFACT_DIR}/determinism-matrix.json"
BENCH_FILE="${ARTIFACT_DIR}/gas-benchmark-envelope.json"
TEST_SUMMARY_FILE="${BUNDLE_DIR}/test-summary.json"
FUZZ_SUMMARY_FILE="${BUNDLE_DIR}/fuzz-summary.json"
VERIFIER_COVERAGE_FILE="${BUNDLE_DIR}/verifier-coverage-report.json"
COMPATIBILITY_FILE="${BUNDLE_DIR}/compatibility-statement.json"
RESIDUAL_RISK_FILE="${BUNDLE_DIR}/residual-risk-statement.json"
ARTIFACTS_MANIFEST_FILE="${BUNDLE_DIR}/artifacts-manifest.json"

log() {
  echo "[aoxcvm-phase3-gate] $*"
}

run_step() {
  local label="$1"
  shift
  log "running: ${label}"
  "$@"
}

sha256_of() {
  local file="$1"
  sha256sum "${file}" | awk '{print $1}'
}

mkdir -p "${ARTIFACT_DIR}"
mkdir -p "${BUNDLE_DIR}"
cd "${REPO_ROOT}"

PROBE_DEBUG_FILE="$(mktemp)"
PROBE_RELEASE_FILE="$(mktemp)"
cleanup() {
  rm -f "${PROBE_DEBUG_FILE}" "${PROBE_RELEASE_FILE}"
}
trap cleanup EXIT

run_step "cargo test -p aoxcvm --test phase3_release_closure" \
  cargo test -p aoxcvm --test phase3_release_closure

run_step "cargo test -p aoxcvm --test phase3_release_closure --release" \
  cargo test -p aoxcvm --test phase3_release_closure --release

run_step "cargo run -p aoxcvm --example determinism_probe" \
  bash -c "cargo run -p aoxcvm --example determinism_probe > ${PROBE_DEBUG_FILE}"
run_step "cargo run -p aoxcvm --example determinism_probe --release" \
  bash -c "cargo run -p aoxcvm --example determinism_probe --release > ${PROBE_RELEASE_FILE}"

DEBUG_HASH="$(tr -d '\n\r' < "${PROBE_DEBUG_FILE}")"
RELEASE_HASH="$(tr -d '\n\r' < "${PROBE_RELEASE_FILE}")"

if [[ -z "${DEBUG_HASH}" || -z "${RELEASE_HASH}" ]]; then
  echo "determinism probe output is empty" >&2
  exit 1
fi

if [[ "${DEBUG_HASH}" != "${RELEASE_HASH}" ]]; then
  echo "debug/release determinism mismatch: ${DEBUG_HASH} vs ${RELEASE_HASH}" >&2
  exit 1
fi

OS_NAME="$(uname -s)"
CPU_ARCH="$(uname -m)"
RUSTC_VERSION="$(rustc --version)"
GIT_SHA="$(git rev-parse HEAD)"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cat > "${MATRIX_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "probe": {
    "debug_hash": "${DEBUG_HASH}",
    "release_hash": "${RELEASE_HASH}",
    "parity": true
  },
  "host_fixture": {
    "os": "${OS_NAME}",
    "cpu_arch": "${CPU_ARCH}",
    "rustc": "${RUSTC_VERSION}"
  },
  "heterogeneous_validator_matrix": [
    {"os": "linux", "cpu": "x86_64", "status": "required"},
    {"os": "linux", "cpu": "aarch64", "status": "required"},
    {"os": "macos", "cpu": "aarch64", "status": "required"},
    {"os": "windows", "cpu": "x86_64", "status": "required"}
  ]
}
JSON

cat > "${BENCH_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "gas_table_justification": {
    "add": 3,
    "storage_write": 200,
    "pq_verify": 5000,
    "invariant": "add < storage_write < pq_verify"
  },
  "dos_cost_envelope": {
    "meter_limit": 10000,
    "expected_out_of_gas_on_pq_verify": true
  },
  "benchmark_diff_gate": {
    "policy": "manual-threshold-review",
    "required_on_release": true
  }
}
JSON

cat > "${TEST_SUMMARY_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "suite": "aoxcvm-phase3-gate",
  "commands": [
    {"cmd": "cargo test -p aoxcvm --test phase3_release_closure", "status": "passed"},
    {"cmd": "cargo test -p aoxcvm --test phase3_release_closure --release", "status": "passed"},
    {"cmd": "cargo run -p aoxcvm --example determinism_probe", "status": "passed"},
    {"cmd": "cargo run -p aoxcvm --example determinism_probe --release", "status": "passed"}
  ]
}
JSON

cat > "${FUZZ_SUMMARY_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "mode": "deterministic-adversarial-closure",
  "coverage": [
    "receipt/proof tamper loops",
    "nonce monotonicity property checks",
    "call-depth and rollback edge cases"
  ],
  "corpus_retention": {
    "status": "pending-ci-nightly-integration",
    "required_for_full_claim": true
  }
}
JSON

cat > "${VERIFIER_COVERAGE_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "scope": "phase3 release closure baseline",
  "coverage_surfaces": [
    "bytecode validation",
    "determinism verifier",
    "invariant verifier",
    "receipt commitment/proof checks"
  ],
  "status": "baseline"
}
JSON

cat > "${COMPATIBILITY_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "statement": "AOXCVM phase3 closure artifacts are generated with deterministic debug/release parity checks and fail-closed verification.",
  "profiles": ["linux-x86_64-required", "linux-aarch64-required", "macos-aarch64-required", "windows-x86_64-required"]
}
JSON

cat > "${RESIDUAL_RISK_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "open_items": [
    "cross-platform matrix must be executed in CI runners for every release candidate",
    "continuous fuzzing corpus retention must be enforced in nightly and release pipelines"
  ],
  "policy": "if any P0/P1 gate lacks machine-verifiable evidence, release is not full"
}
JSON

cat > "${ARTIFACTS_MANIFEST_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "artifacts": [
    {"path": "determinism-matrix.json", "sha256": "$(sha256_of "${MATRIX_FILE}")"},
    {"path": "gas-benchmark-envelope.json", "sha256": "$(sha256_of "${BENCH_FILE}")"},
    {"path": "evidence-bundle/test-summary.json", "sha256": "$(sha256_of "${TEST_SUMMARY_FILE}")"},
    {"path": "evidence-bundle/fuzz-summary.json", "sha256": "$(sha256_of "${FUZZ_SUMMARY_FILE}")"},
    {"path": "evidence-bundle/verifier-coverage-report.json", "sha256": "$(sha256_of "${VERIFIER_COVERAGE_FILE}")"},
    {"path": "evidence-bundle/compatibility-statement.json", "sha256": "$(sha256_of "${COMPATIBILITY_FILE}")"},
    {"path": "evidence-bundle/residual-risk-statement.json", "sha256": "$(sha256_of "${RESIDUAL_RISK_FILE}")"}
  ]
}
JSON

log "phase-3 gate completed; artifacts written to ${ARTIFACT_DIR}"
