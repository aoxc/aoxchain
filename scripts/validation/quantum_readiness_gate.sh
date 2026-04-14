#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRICT="${AOXC_QUANTUM_GATE_STRICT:-0}"
ARTIFACT_DIR="${AOXC_QUANTUM_GATE_ARTIFACT_DIR:-${REPO_ROOT}/artifacts/quantum-gate}"
READ_FILE="${REPO_ROOT}/READ.md"
ROADMAP_FILE="${REPO_ROOT}/ROADMAP.md"
SCOPE_FILE="${REPO_ROOT}/SCOPE.md"
CHECKLIST_FILE="${REPO_ROOT}/docs/src/architecture/PQ_AUTHORITY_IMPLEMENTATION_CHECKLIST.md"

required_files=(
  "${READ_FILE}"
  "${ROADMAP_FILE}"
  "${SCOPE_FILE}"
)

required_read_sections=(
  "## 2) Core Invariants"
  "## 4) Cryptographic Profile Contract"
  "## 6) Required Gate Baseline"
)

required_roadmap_sections=(
  "## Hard Constraints (Non-Negotiable)"
  "## Phase 3 — Validation Kernel (PQ-Primary, Hybrid-Capable)"
  "## Phase 5 — Testnet Hardening and Promotion Gate"
  "## Phase 6 — Mainnet Activation Decision (Governed)"
)

required_quantum_files=(
  "${REPO_ROOT}/docs/quantum/CLOSE_PLAN.md"
  "${REPO_ROOT}/docs/quantum/CLOSE_TRACKS.md"
  "${REPO_ROOT}/docs/quantum/CUTOVER_RUNBOOK.md"
  "${REPO_ROOT}/docs/quantum/EVIDENCE_PACKAGE.md"
  "${CHECKLIST_FILE}"
)

required_quantum_plan_sections=(
  "## Entry Conditions"
  "## Acceptance Rule"
  "## Non-Goals"
)

required_quantum_tracks_sections=(
  "## 1) Kernel Policy Closure"
  "## 2) Network and Handshake Closure"
  "## 5) Evidence and Gate Closure"
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

for file in "${required_quantum_files[@]}"; do
  if [[ ! -f "${file}" ]]; then
    log "missing required quantum document: ${file}"
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

for section in "${required_quantum_plan_sections[@]}"; do
  if ! grep -Fq "${section}" "${REPO_ROOT}/docs/quantum/CLOSE_PLAN.md"; then
    log "missing quantum close plan section: ${section}"
    missing=1
  fi
done

for section in "${required_quantum_tracks_sections[@]}"; do
  if ! grep -Fq "${section}" "${REPO_ROOT}/docs/quantum/CLOSE_TRACKS.md"; then
    log "missing quantum close tracks section: ${section}"
    missing=1
  fi
done

roadmap_phase_count="$(grep -Ec '^## Phase [0-9]+ — ' "${ROADMAP_FILE}" || true)"
if [[ "${roadmap_phase_count}" -lt 6 ]]; then
  log "roadmap has insufficient phase sections: ${roadmap_phase_count}"
  missing=1
fi

mkdir -p "${ARTIFACT_DIR}"
SUMMARY_FILE="${ARTIFACT_DIR}/summary.json"
CHECKLIST_SUMMARY_FILE="${ARTIFACT_DIR}/pq-checklist-status.json"

python3 - "${CHECKLIST_FILE}" "${CHECKLIST_SUMMARY_FILE}" <<'PY'
import json
import re
import sys
from pathlib import Path

checklist_path = Path(sys.argv[1])
output_path = Path(sys.argv[2])
lines = checklist_path.read_text(encoding="utf-8").splitlines()

section = "Unsectioned"
sections = {}
pattern = re.compile(r"^\s*-\s+\[(x| |\-)\]\s+(.*)$", re.IGNORECASE)

status_map = {
    "x": "completed",
    " ": "pending",
    "-": "deferred",
}

for raw in lines:
    if raw.startswith("## "):
        section = raw[3:].strip()
        sections.setdefault(
            section, {"completed": 0, "pending": 0, "deferred": 0, "items": []}
        )
        continue
    m = pattern.match(raw)
    if not m:
        continue
    marker = m.group(1).lower()
    text = m.group(2).strip()
    status = status_map[marker]
    sections.setdefault(
        section, {"completed": 0, "pending": 0, "deferred": 0, "items": []}
    )
    sections[section][status] += 1
    sections[section]["items"].append({"status": status, "text": text})

totals = {"completed": 0, "pending": 0, "deferred": 0}
for data in sections.values():
    totals["completed"] += data["completed"]
    totals["pending"] += data["pending"]
    totals["deferred"] += data["deferred"]

result = {
    "source": str(checklist_path),
    "totals": totals,
    "sections": sections,
}

output_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
PY

pending_checklist_items="$(python3 - "${CHECKLIST_SUMMARY_FILE}" <<'PY'
import json
import sys
from pathlib import Path
data = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(data["totals"]["pending"])
PY
)"

if [[ "${pending_checklist_items}" -gt 0 ]]; then
  warn=1
  log "checklist has pending items: ${pending_checklist_items}"
fi

cat > "${SUMMARY_FILE}" <<JSON
{
  "suite": "quantum-readiness-gate",
  "strict_mode": ${STRICT},
  "timestamp_utc": "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)",
  "surfaces": {
    "read_file": "READ.md",
    "roadmap_file": "ROADMAP.md",
    "roadmap_phase_count": ${roadmap_phase_count},
    "quantum_documents": [
      "docs/quantum/CLOSE_PLAN.md",
      "docs/quantum/CLOSE_TRACKS.md",
      "docs/quantum/CUTOVER_RUNBOOK.md",
      "docs/quantum/EVIDENCE_PACKAGE.md",
      "docs/src/architecture/PQ_AUTHORITY_IMPLEMENTATION_CHECKLIST.md"
    ],
    "checklist_status_file": "artifacts/quantum-gate/pq-checklist-status.json",
    "checklist_pending_items": ${pending_checklist_items}
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

if [[ "${STRICT}" -eq 1 && "${warn}" -ne 0 ]]; then
  log "FAILED: strict mode enabled and checklist still has pending items"
  exit 1
fi

log "PASS: quantum readiness surfaces are structurally valid"
log "artifact: ${SUMMARY_FILE}"
log "artifact: ${CHECKLIST_SUMMARY_FILE}"
