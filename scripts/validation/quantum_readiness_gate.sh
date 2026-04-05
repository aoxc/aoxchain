#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRICT="${AOXC_QUANTUM_GATE_STRICT:-0}"
ARTIFACT_DIR="${AOXC_QUANTUM_GATE_ARTIFACT_DIR:-${REPO_ROOT}/artifacts/quantum-gate}"
READ_FILE="${REPO_ROOT}/READ.md"
ROADMAP_FILE="${REPO_ROOT}/ROADMAP.md"
SCOPE_FILE="${REPO_ROOT}/SCOPE.md"

required_files=(
  "${READ_FILE}"
  "${ROADMAP_FILE}"
  "${SCOPE_FILE}"
)

required_read_sections=(
  "## 2) Core Invariants"
  "## 4) Readiness Contract"
  "## 6) Program Trajectory"
)

required_roadmap_sections=(
  "## Phase 2 — Production-Grade Testnet Baseline"
  "## Phase 3 — Cryptographic Agility Activation (Hybrid)"
  "## Phase 4 — PQ-Resilient Mainnet Readiness Gate"
  "## Non-Negotiable Program Rules"
)

missing=0
warn=0

log() {
  echo "[quantum-readiness-gate] $*"
}

for file in "${required_files[@]}"; do
  if [[ ! -f "${file}" ]]; then
    log "missing required file: ${file}"
    missing=1
  fi
done

for section in "${required_read_sections[@]}"; do
  if ! grep -Fq "${section}" "${READ_FILE}"; then
    log "missing READ section: ${section}"
    missing=1
  fi
done

for section in "${required_roadmap_sections[@]}"; do
  if ! grep -Fq "${section}" "${ROADMAP_FILE}"; then
    log "missing roadmap phase section: ${section}"
    missing=1
  fi
done

roadmap_phase_count="$(grep -Ec '^## Phase [0-9]+ — ' "${ROADMAP_FILE}" || true)"
if [[ "${roadmap_phase_count}" -lt 4 ]]; then
  log "roadmap has insufficient phase sections: ${roadmap_phase_count}"
  missing=1
fi

mkdir -p "${ARTIFACT_DIR}"
SUMMARY_FILE="${ARTIFACT_DIR}/summary.json"

cat > "${SUMMARY_FILE}" <<JSON
{
  "suite": "quantum-readiness-gate",
  "strict_mode": ${STRICT},
  "timestamp_utc": "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)",
  "surfaces": {
    "read_file": "READ.md",
    "roadmap_file": "ROADMAP.md",
    "roadmap_phase_count": ${roadmap_phase_count}
  },
  "status": {
    "missing_required_surfaces": ${missing},
    "strict_warning_present": ${warn}
  }
}
JSON

if [[ "${missing}" -ne 0 ]]; then
  log "FAILED: required quantum readiness surfaces are missing"
  exit 1
fi

log "PASS: quantum readiness surfaces are structurally valid"
log "artifact: ${SUMMARY_FILE}"
