#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
AOXC Autonomy Control Plane (experimental)

Usage:
  scripts/autonomy_control_plane.sh <command>

Commands:
  plan            Print the implementation blueprint for full autonomous operation.
  doctor          Validate local prerequisites for autonomy-oriented operations.
  left-mainnet    Print the mainnet-only sidebar command profile.
  right-multi     Print the integrated multi-network sidebar command profile.
USAGE
}

plan() {
  cat <<'PLAN'
=== AOXC Full Autonomous System Blueprint ===

1) Mainnet-first operating mode (left sidebar)
   - Wallet address lifecycle: create/import/export/verify.
   - Core transactional flows: transfer, stake, undelegate, claim.
   - Safety rails: fee estimation, preflight simulation, policy checks.
   - AI operator mode: intent -> reviewed transaction plan -> signed execution.

2) Integrated multi-network mode (right sidebar)
   - Unified environment switch: mainnet/devnet/testnet.
   - Cross-environment command parity for all operational primitives.
   - Cross-chain interaction layer with route validation and replay protection.
   - Central observability panel for health, latency, peers, and finality.

3) Automation control loop
   - Policy engine validates every intent before execution.
   - Scheduler handles periodic tasks: status probes, restake, evidence export.
   - Risk controller can pause automation by environment or operation class.

4) Delivery gates
   - Deterministic CLI surface with Make targets and scripts.
   - Integration tests for transfer/stake/bridge across environments.
   - Production readiness gate for mainnet release candidate.
PLAN
}

doctor() {
  local missing=0
  for tool in bash make cargo git; do
    if ! command -v "$tool" >/dev/null 2>&1; then
      echo "[missing] $tool"
      missing=1
    else
      echo "[ok] $tool"
    fi
  done

  for script in scripts/aoxc_easy.sh scripts/network_env_daemon.sh scripts/network_stack.sh; do
    if [[ -x "$script" ]]; then
      echo "[ok] executable: $script"
    elif [[ -f "$script" ]]; then
      echo "[warn] not executable: $script"
    else
      echo "[missing] $script"
      missing=1
    fi
  done

  if [[ "$missing" -eq 1 ]]; then
    echo "Autonomy doctor finished with missing prerequisites."
    exit 1
  fi

  echo "Autonomy doctor passed."
}

left_mainnet() {
  cat <<'LEFT'
=== Left Sidebar: Mainnet-Only Easy Operations ===
make ops-start-mainnet
make ops-status-mainnet
make ops-logs-mainnet
make ops-stop-mainnet

# core economic actions via CLI
$AOXC_BIN_PATH wallet address create --network mainnet
$AOXC_BIN_PATH tx transfer --network mainnet <to> <amount>
$AOXC_BIN_PATH tx stake --network mainnet <validator> <amount>
LEFT
}

right_multi() {
  cat <<'RIGHT'
=== Right Sidebar: Integrated Full System ===
make ops-dashboard
make ops-status-mainnet
make ops-status-testnet
make ops-status-devnet
make net-dual-status

# execution matrix (policy-gated)
$AOXC_BIN_PATH tx transfer --network {mainnet|testnet|devnet}
$AOXC_BIN_PATH tx stake --network {mainnet|testnet|devnet}
$AOXC_BIN_PATH bridge send --from <network> --to <network>
RIGHT
}

main() {
  local cmd="${1:-}"
  case "$cmd" in
    plan) plan ;;
    doctor) doctor ;;
    left-mainnet) left_mainnet ;;
    right-multi) right_multi ;;
    -h|--help|help|"") usage ;;
    *)
      echo "Unknown command: $cmd" >&2
      usage
      exit 2
      ;;
  esac
}

main "$@"
