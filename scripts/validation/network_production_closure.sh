#!/usr/bin/env bash
set -euo pipefail

SCENARIO="${SCENARIO:-all}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      SCENARIO="$2"
      shift 2
      ;;
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

ARTIFACT_DIR="${ARTIFACT_DIR:-artifacts/network-production-closure}"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
DURATION_MINUTES="${DURATION_MINUTES:-30}"
SNAPSHOT_FILE="${SNAPSHOT_FILE:-${ARTIFACT_DIR}/snapshot-recovery.json}"
COMPAT_FILE="${COMPAT_FILE:-${ARTIFACT_DIR}/compat-matrix.json}"
ALERT_FILE="${ALERT_FILE:-${ARTIFACT_DIR}/alert-rules.md}"
mkdir -p "${ARTIFACT_DIR}"

run_multi_host() {
  ./scripts/validation/multi_host_validation.sh
}

run_partition_faults() {
  cat > "${ARTIFACT_DIR}/fault-injection-plan.md" <<PLAN
# AOXC Fault Injection Plan

- scenario: partition
- scenario: delay
- scenario: drop
- scenario: restart
- scenario: timeout
- requirement: collect node-run output, health output, and operator timeline
- result file: ${ARTIFACT_DIR}/fault-injection-results.json
PLAN

  cat > "${ARTIFACT_DIR}/fault-injection-results.json" <<JSON
{
  "status": "planned",
  "scenarios": ["partition", "delay", "drop", "restart", "timeout"],
  "evidence_requirement": "attach host-level logs and remediation notes before release"
}
JSON
}

run_recovery() {
  ${AOXC_BIN} compat-matrix > "${COMPAT_FILE}"
  cat > "${SNAPSHOT_FILE}" <<JSON
{
  "status": "scripted",
  "steps": [
    "bootstrap source node",
    "export snapshot and journal",
    "wipe target runtime home",
    "import snapshot",
    "run node-health and runtime-status",
    "verify compat-matrix before rejoin"
  ],
  "verification": [
    "height monotonic after restore",
    "state root matches exported snapshot",
    "node rejoins without protocol mismatch"
  ]
}
JSON
}

run_soak() {
  ${AOXC_BIN} production-audit --format json > "${ARTIFACT_DIR}/production-audit.json"
  ${AOXC_BIN} runtime-status --format json > "${ARTIFACT_DIR}/runtime-status.json"
  cat > "${ARTIFACT_DIR}/soak-plan.json" <<JSON
{
  "status": "scripted",
  "duration_minutes": ${DURATION_MINUTES},
  "required_metrics": [
    "block progression",
    "peer count",
    "rpc errors",
    "process restarts",
    "readiness score"
  ],
  "artifacts": [
    "production-audit.json",
    "runtime-status.json",
    "telemetry-snapshot.json"
  ]
}
JSON

  cat > "${ARTIFACT_DIR}/telemetry-snapshot.json" <<JSON
{
  "status": "baseline",
  "alerts_required": [
    "validator stalled",
    "peer count below quorum",
    "rpc readiness score degraded",
    "snapshot recovery failed"
  ]
}
JSON

  cat > "${ARTIFACT_DIR}/aoxhub-rollout.json" <<JSON
{
  "status": "planned",
  "surfaces": ["mainnet", "testnet", "aoxhub"],
  "requirements": [
    "hub api parity against mainnet/testnet baselines",
    "bridge and rpc health verified before promotion",
    "operator rollback contact and ownership recorded"
  ]
}
JSON

  cat > "${ALERT_FILE}" <<'RULES'
# AOXC Alert Rules

- Critical: no finalized progress within 3 block windows.
- Critical: peer count drops below configured quorum threshold.
- High: readiness score below 85 for more than 5 minutes.
- High: snapshot restore or state-sync rehearsal exits non-zero.
- Warning: RPC rate limit surge indicates abusive traffic or client regression.
RULES
}

case "${SCENARIO}" in
  all)
    run_multi_host
    run_partition_faults
    run_recovery
    run_soak
    ;;
  multi-host)
    run_multi_host
    ;;
  partition|fault|faults)
    run_partition_faults
    ;;
  recovery|snapshot)
    run_recovery
    ;;
  soak)
    run_soak
    ;;
  *)
    echo "unknown scenario: ${SCENARIO}" >&2
    exit 2
    ;;
esac

echo "[done] network production closure artifacts in ${ARTIFACT_DIR}"
