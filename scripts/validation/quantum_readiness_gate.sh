#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRICT="${AOXC_QUANTUM_GATE_STRICT:-0}"
ARTIFACT_DIR="${AOXC_QUANTUM_GATE_ARTIFACT_DIR:-${REPO_ROOT}/artifacts/quantum-gate}"
CHECKLIST_FILE="${REPO_ROOT}/QUANTUM_CHECKLIST.md"
ROADMAP_FILE="${REPO_ROOT}/QUANTUM_ROADMAP.md"
SCOPE_FILE="${REPO_ROOT}/SCOPE.md"

required_files=(
  "${CHECKLIST_FILE}"
  "${ROADMAP_FILE}"
  "${SCOPE_FILE}"
)

required_checklist_sections=(
  "## 1) Program Governance and Threat Modeling"
  "## 3) Block Structure Upgrades (First Implementation Priority)"
  "## 5) Deterministic VM (Special-Purpose Hardening)"
  "## 6) P2P Networking and Handshake"
  "## 7) Consensus and Governance Controls"
  "## 11) Verification and Audit"
  "## 12) Release and Production Closure"
)

required_roadmap_sections=(
  "## Phase 0 — Threat Model and Baseline Inventory (Foundation)"
  "## Phase 1 — Block Structure Modernization (First protocol target)"
  "## Phase 2 — Special-Purpose Deterministic VM for PQ Era"
  "## Phase 3 — Network and Handshake Cryptography"
  "## Phase 4 — Consensus and Governance Controls"
  "## Phase 5 — AI-Assisted Security and Runtime Risk Controls"
  "## Phase 6 — Verification, Audit, and Production Closure"
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

for section in "${required_checklist_sections[@]}"; do
  if ! grep -Fq "${section}" "${CHECKLIST_FILE}"; then
    log "missing checklist section: ${section}"
    missing=1
  fi
done

for section in "${required_roadmap_sections[@]}"; do
  if ! grep -Fq "${section}" "${ROADMAP_FILE}"; then
    log "missing roadmap phase section: ${section}"
    missing=1
  fi
done

total_items="$(grep -Ec '^- \[[ xX]\] ' "${CHECKLIST_FILE}" || true)"
completed_items="$(grep -Ec '^- \[[xX]\] ' "${CHECKLIST_FILE}" || true)"
incomplete_items=$(( total_items - completed_items ))

if [[ "${total_items}" -eq 0 ]]; then
  log "no checklist items detected in ${CHECKLIST_FILE}"
  missing=1
fi

if [[ "${incomplete_items}" -gt 0 ]]; then
  log "checklist has ${incomplete_items}/${total_items} incomplete items"
  warn=1
fi

mkdir -p "${ARTIFACT_DIR}"
SUMMARY_FILE="${ARTIFACT_DIR}/summary.json"

cat > "${SUMMARY_FILE}" <<JSON
{
  "suite": "quantum-readiness-gate",
  "strict_mode": ${STRICT},
  "timestamp_utc": "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)",
  "checklist": {
    "file": "QUANTUM_CHECKLIST.md",
    "total_items": ${total_items},
    "completed_items": ${completed_items},
    "incomplete_items": ${incomplete_items}
  },
  "status": {
    "missing_required_surfaces": ${missing},
    "incomplete_items_present": ${warn}
  }
}
JSON

if [[ "${missing}" -ne 0 ]]; then
  log "FAILED: required quantum readiness surfaces are missing"
  exit 1
fi

if [[ "${warn}" -ne 0 && "${STRICT}" == "1" ]]; then
  log "FAILED (strict mode): checklist includes incomplete items"
  exit 1
fi

if [[ "${warn}" -ne 0 ]]; then
  log "WARNING: checklist includes incomplete items (non-strict mode)"
fi

log "PASS: quantum readiness surfaces are structurally valid"
log "artifact: ${SUMMARY_FILE}"
