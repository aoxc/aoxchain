#!/usr/bin/env bash
set -euo pipefail

# AOXCVM production closure gate.
# Full closure requires all gate classes to pass together:
#   1) test gate
#   2) audit gate
#   3) rehearsal gate
#   4) evidence gate

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARTIFACT_DIR="${AOXCVM_ARTIFACT_DIR:-${ROOT_DIR}/artifacts/aoxcvm-phase3}"
SUMMARY_FILE="${ARTIFACT_DIR}/production-closure-summary.json"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
GIT_SHA="$(git -C "${ROOT_DIR}" rev-parse HEAD)"

log() {
  echo "[aoxcvm-production-closure] $*"
}

run_gate() {
  local gate_name="$1"
  shift
  log "running ${gate_name} gate: $*"
  if "$@"; then
    log "PASS ${gate_name}"
    return 0
  fi
  log "FAIL ${gate_name}"
  return 1
}

mkdir -p "${ARTIFACT_DIR}"
cd "${ROOT_DIR}"

test_status="failed"
audit_status="failed"
rehearsal_status="failed"
evidence_status="failed"
overall_status="failed"

if run_gate "test" ./scripts/validation/aoxcvm_phase3_gate.sh; then
  test_status="passed"
fi

if run_gate "audit" cargo audit; then
  audit_status="passed"
fi

if run_gate "rehearsal" ./scripts/validation/os_compatibility_gate.sh; then
  rehearsal_status="passed"
fi

if run_gate "evidence" test -f "${ARTIFACT_DIR}/evidence-bundle/artifacts-manifest.json"; then
  evidence_status="passed"
fi

if [[ "${test_status}" == "passed" \
  && "${audit_status}" == "passed" \
  && "${rehearsal_status}" == "passed" \
  && "${evidence_status}" == "passed" ]]; then
  overall_status="passed"
  log "PASS: full production closure achieved (test/audit/rehearsal/evidence)."
else
  log "FAILED: full production closure requires all four gate classes to pass."
fi

cat > "${SUMMARY_FILE}" <<JSON
{
  "generated_at_utc": "${TIMESTAMP}",
  "git_sha": "${GIT_SHA}",
  "suite": "aoxcvm-production-closure-gate",
  "gates": {
    "test": "${test_status}",
    "audit": "${audit_status}",
    "rehearsal": "${rehearsal_status}",
    "evidence": "${evidence_status}"
  },
  "overall": "${overall_status}",
  "policy": "full closure requires test/audit/rehearsal/evidence to all be PASS"
}
JSON

log "wrote summary: ${SUMMARY_FILE}"

if [[ "${overall_status}" != "passed" ]]; then
  exit 1
fi
