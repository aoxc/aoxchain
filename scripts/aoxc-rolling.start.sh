#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
#
# Purpose:
#   Provision and supervise a persistent local AOXC testnet/devnet surface that
#   approximates a controlled operator-grade environment with deterministic
#   filesystem layout, deterministic port planning, and hardened audit outputs.
#
# Security and operational objectives:
#   - Fail closed on missing required inputs.
#   - Avoid dead code, implicit best-effort behavior, and silent parsing fallbacks.
#   - Emit durable audit artifacts for operator validation and incident review.
#   - Use strong per-node passwords instead of predictable static strings.
#   - Preserve deterministic path structure for repeatable local orchestration.
set -Eeuo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AOXC_Q_HOME="${AOXC_Q_HOME:-/mnt/xdbx/aoxc}"
AOXC_Q_ENV="${AOXC_Q_ENV:-testnet}"
AOXC_Q_PROFILE="${AOXC_Q_PROFILE:-testnet}"
AOXC_Q_MODE="${AOXC_Q_MODE:-local}"
AOXC_Q_NODE_COUNT="${AOXC_Q_NODE_COUNT:-7}"
AOXC_Q_REAL_TESTNET="${AOXC_Q_REAL_TESTNET:-0}"
AOXC_Q_ROUNDS="${AOXC_Q_ROUNDS:-200}"
AOXC_Q_BLOCK_INTERVAL_SECS="${AOXC_Q_BLOCK_INTERVAL_SECS:-6}"
AOXC_Q_SLEEP_SECS="${AOXC_Q_SLEEP_SECS:-3}"
AOXC_Q_SLEEP_MIN_SECS="${AOXC_Q_SLEEP_MIN_SECS:-3}"
AOXC_Q_SLEEP_MAX_SECS="${AOXC_Q_SLEEP_MAX_SECS:-3}"
AOXC_Q_HEALTH_INTERVAL_SECS="${AOXC_Q_HEALTH_INTERVAL_SECS:-3}"
AOXC_Q_FORCE="${AOXC_Q_FORCE:-0}"
AOXC_Q_ACTION="${AOXC_Q_ACTION:-up}"
AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE="${AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE:-250000}"
AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE="${AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE:-500000}"

AOXC_Q_RPC_BASE_PORT="${AOXC_Q_RPC_BASE_PORT:-18540}"
AOXC_Q_P2P_BASE_PORT="${AOXC_Q_P2P_BASE_PORT:-19540}"
AOXC_Q_METRICS_BASE_PORT="${AOXC_Q_METRICS_BASE_PORT:-20540}"
AOXC_Q_ADMIN_BASE_PORT="${AOXC_Q_ADMIN_BASE_PORT:-21540}"
AOXC_Q_VALIDATE_GENESIS="${AOXC_Q_VALIDATE_GENESIS:-1}"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

AOXC rolling local supervisor.

Actions:
  up (default)     provision and start node loops
  provision        provision only
  start            start loops for an existing provisioned root
  stop             stop loops for an existing provisioned root
  restart          stop then start
  status           show per-node process and chain status

Options:
  --action <name>      one of: up|provision|start|stop|restart|status
  --home <path>        base output root (default: ${AOXC_Q_HOME})
  --env <name>         configs/environments/<name> source (default: ${AOXC_Q_ENV})
  --profile <name>     AOXC profile for bootstrap (default: ${AOXC_Q_PROFILE})
  --mode <name>        run mode label: local|public (default: ${AOXC_Q_MODE})
  --nodes <n>          node count (default: ${AOXC_Q_NODE_COUNT}; minimum: 7)
  --real-testnet       enforce production-like full testnet sizing/policy defaults
  --rounds <n>         rounds per node-run cycle (default: ${AOXC_Q_ROUNDS})
  --block-interval-secs <n> block production interval in seconds for node-run (default: ${AOXC_Q_BLOCK_INTERVAL_SECS}; range: 2..600)
  --sleep-secs <n>     fixed sleep between daemon cycles
  --sleep-min-secs <n> minimum daemon-loop sleep
  --sleep-max-secs <n> maximum daemon-loop sleep
  --health-interval-secs <n> monitor/recovery loop interval
  --operator-balance <n>  initial treasury transfer amount for each operator wallet
  --validator-balance <n> initial treasury transfer amount for each validator wallet
  --rpc-base-port <n>      base RPC port
  --p2p-base-port <n>      base P2P port
  --metrics-base-port <n>  base metrics port
  --admin-base-port <n>    base admin port
  --skip-genesis-validate  skip sha256/genesis consistency gate (not recommended)
  --no-start           alias for --action provision
  --force              recreate target root during provision
  -h, --help           show this help

Environment overrides:
  AOXC_Q_HOME, AOXC_Q_ENV, AOXC_Q_PROFILE, AOXC_Q_MODE, AOXC_Q_NODE_COUNT,
  AOXC_Q_REAL_TESTNET,
  AOXC_Q_ROUNDS, AOXC_Q_BLOCK_INTERVAL_SECS, AOXC_Q_SLEEP_SECS, AOXC_Q_SLEEP_MIN_SECS,
  AOXC_Q_SLEEP_MAX_SECS, AOXC_Q_FORCE, AOXC_Q_ACTION, AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE,
  AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE,
  AOXC_Q_HEALTH_INTERVAL_SECS,
  AOXC_Q_RPC_BASE_PORT, AOXC_Q_P2P_BASE_PORT, AOXC_Q_METRICS_BASE_PORT,
  AOXC_Q_ADMIN_BASE_PORT, AOXC_Q_VALIDATE_GENESIS
USAGE
}

log_info() { printf '[aoxc-q][info] %s\n' "$*"; }
log_warn() { printf '[aoxc-q][warn] %s\n' "$*" >&2; }
log_error() { printf '[aoxc-q][error] %s\n' "$*" >&2; }

die() {
  local message="$1"
  local code="${2:-1}"
  log_error "${message}"
  exit "${code}"
}

require_uint() {
  local value="$1"
  local name="$2"
  [[ "${value}" =~ ^[0-9]+$ ]] || die "${name} must be an unsigned integer (got: ${value})" 2
}

ensure_port_in_range() {
  local value="$1"
  local name="$2"
  if (( value < 1 || value > 65535 )); then
    die "${name} must be in range 1..65535 (got: ${value})" 2
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --action) AOXC_Q_ACTION="$2"; shift 2 ;;
    --action=*) AOXC_Q_ACTION="${1#*=}"; shift ;;
    --home) AOXC_Q_HOME="$2"; shift 2 ;;
    --home=*) AOXC_Q_HOME="${1#*=}"; shift ;;
    --env) AOXC_Q_ENV="$2"; shift 2 ;;
    --env=*) AOXC_Q_ENV="${1#*=}"; shift ;;
    --profile) AOXC_Q_PROFILE="$2"; shift 2 ;;
    --profile=*) AOXC_Q_PROFILE="${1#*=}"; shift ;;
    --mode) AOXC_Q_MODE="$2"; shift 2 ;;
    --mode=*) AOXC_Q_MODE="${1#*=}"; shift ;;
    --nodes) AOXC_Q_NODE_COUNT="$2"; shift 2 ;;
    --nodes=*) AOXC_Q_NODE_COUNT="${1#*=}"; shift ;;
    --real-testnet) AOXC_Q_REAL_TESTNET=1; shift ;;
    --rounds) AOXC_Q_ROUNDS="$2"; shift 2 ;;
    --rounds=*) AOXC_Q_ROUNDS="${1#*=}"; shift ;;
    --block-interval-secs) AOXC_Q_BLOCK_INTERVAL_SECS="$2"; shift 2 ;;
    --block-interval-secs=*) AOXC_Q_BLOCK_INTERVAL_SECS="${1#*=}"; shift ;;
    --sleep-secs) AOXC_Q_SLEEP_SECS="$2"; shift 2 ;;
    --sleep-secs=*) AOXC_Q_SLEEP_SECS="${1#*=}"; shift ;;
    --sleep-min-secs) AOXC_Q_SLEEP_MIN_SECS="$2"; shift 2 ;;
    --sleep-min-secs=*) AOXC_Q_SLEEP_MIN_SECS="${1#*=}"; shift ;;
    --sleep-max-secs) AOXC_Q_SLEEP_MAX_SECS="$2"; shift 2 ;;
    --sleep-max-secs=*) AOXC_Q_SLEEP_MAX_SECS="${1#*=}"; shift ;;
    --health-interval-secs) AOXC_Q_HEALTH_INTERVAL_SECS="$2"; shift 2 ;;
    --health-interval-secs=*) AOXC_Q_HEALTH_INTERVAL_SECS="${1#*=}"; shift ;;
    --operator-balance) AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE="$2"; shift 2 ;;
    --operator-balance=*) AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE="${1#*=}"; shift ;;
    --validator-balance) AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE="$2"; shift 2 ;;
    --validator-balance=*) AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE="${1#*=}"; shift ;;
    --rpc-base-port) AOXC_Q_RPC_BASE_PORT="$2"; shift 2 ;;
    --rpc-base-port=*) AOXC_Q_RPC_BASE_PORT="${1#*=}"; shift ;;
    --p2p-base-port) AOXC_Q_P2P_BASE_PORT="$2"; shift 2 ;;
    --p2p-base-port=*) AOXC_Q_P2P_BASE_PORT="${1#*=}"; shift ;;
    --metrics-base-port) AOXC_Q_METRICS_BASE_PORT="$2"; shift 2 ;;
    --metrics-base-port=*) AOXC_Q_METRICS_BASE_PORT="${1#*=}"; shift ;;
    --admin-base-port) AOXC_Q_ADMIN_BASE_PORT="$2"; shift 2 ;;
    --admin-base-port=*) AOXC_Q_ADMIN_BASE_PORT="${1#*=}"; shift ;;
    --skip-genesis-validate) AOXC_Q_VALIDATE_GENESIS=0; shift ;;
    --no-start) AOXC_Q_ACTION="provision"; shift ;;
    --force) AOXC_Q_FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "Unknown argument: $1" 2 ;;
  esac
done

require_uint "${AOXC_Q_NODE_COUNT}" "AOXC_Q_NODE_COUNT"
require_uint "${AOXC_Q_ROUNDS}" "AOXC_Q_ROUNDS"
require_uint "${AOXC_Q_BLOCK_INTERVAL_SECS}" "AOXC_Q_BLOCK_INTERVAL_SECS"
require_uint "${AOXC_Q_SLEEP_SECS}" "AOXC_Q_SLEEP_SECS"
require_uint "${AOXC_Q_SLEEP_MIN_SECS}" "AOXC_Q_SLEEP_MIN_SECS"
require_uint "${AOXC_Q_SLEEP_MAX_SECS}" "AOXC_Q_SLEEP_MAX_SECS"
require_uint "${AOXC_Q_HEALTH_INTERVAL_SECS}" "AOXC_Q_HEALTH_INTERVAL_SECS"
require_uint "${AOXC_Q_RPC_BASE_PORT}" "AOXC_Q_RPC_BASE_PORT"
require_uint "${AOXC_Q_P2P_BASE_PORT}" "AOXC_Q_P2P_BASE_PORT"
require_uint "${AOXC_Q_METRICS_BASE_PORT}" "AOXC_Q_METRICS_BASE_PORT"
require_uint "${AOXC_Q_ADMIN_BASE_PORT}" "AOXC_Q_ADMIN_BASE_PORT"
require_uint "${AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE}" "AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE"
require_uint "${AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE}" "AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE"

if (( AOXC_Q_NODE_COUNT < 7 )); then
  die "AOXC_Q_NODE_COUNT must be >= 7 for production-like persistent topology." 2
fi
if [[ "${AOXC_Q_REAL_TESTNET}" != "0" && "${AOXC_Q_REAL_TESTNET}" != "1" ]]; then
  die "AOXC_Q_REAL_TESTNET must be 0 or 1." 2
fi
if (( AOXC_Q_SLEEP_MIN_SECS < 1 )); then
  die "AOXC_Q_SLEEP_MIN_SECS must be >= 1." 2
fi
if (( AOXC_Q_SLEEP_MAX_SECS < AOXC_Q_SLEEP_MIN_SECS )); then
  die "AOXC_Q_SLEEP_MAX_SECS must be >= AOXC_Q_SLEEP_MIN_SECS." 2
fi
if (( AOXC_Q_HEALTH_INTERVAL_SECS < 1 )); then
  die "AOXC_Q_HEALTH_INTERVAL_SECS must be >= 1." 2
fi
if (( AOXC_Q_BLOCK_INTERVAL_SECS < 2 || AOXC_Q_BLOCK_INTERVAL_SECS > 600 )); then
  die "AOXC_Q_BLOCK_INTERVAL_SECS must be in range 2..600." 2
fi
ensure_port_in_range "${AOXC_Q_RPC_BASE_PORT}" "AOXC_Q_RPC_BASE_PORT"
ensure_port_in_range "${AOXC_Q_P2P_BASE_PORT}" "AOXC_Q_P2P_BASE_PORT"
ensure_port_in_range "${AOXC_Q_METRICS_BASE_PORT}" "AOXC_Q_METRICS_BASE_PORT"
ensure_port_in_range "${AOXC_Q_ADMIN_BASE_PORT}" "AOXC_Q_ADMIN_BASE_PORT"

case "${AOXC_Q_ACTION}" in
  up|provision|start|stop|restart|status) ;;
  *) die "Invalid --action value: ${AOXC_Q_ACTION}" 2 ;;
esac

case "${AOXC_Q_MODE}" in
  local|public) ;;
  *) die "Invalid --mode value: ${AOXC_Q_MODE} (expected: local|public)" 2 ;;
esac

if [[ "${AOXC_Q_REAL_TESTNET}" == "1" ]]; then
  if (( AOXC_Q_NODE_COUNT < 21 )); then
    log_warn "real testnet requested: bumping node count from ${AOXC_Q_NODE_COUNT} to 21"
    AOXC_Q_NODE_COUNT=21
  fi
  if [[ "${AOXC_Q_MODE}" != "public" ]]; then
    log_warn "real testnet requested: overriding mode=${AOXC_Q_MODE} -> public"
    AOXC_Q_MODE="public"
  fi
fi

TARGET_ROOT="${AOXC_Q_HOME%/}/aoxc-rolling-${AOXC_Q_ENV}-${AOXC_Q_NODE_COUNT}n"
SOURCE_ROOT="${REPO_ROOT}/configs/environments/${AOXC_Q_ENV}"
SOURCE_TOPOLOGY_ROOT="${SOURCE_ROOT}/topology"

required_source_files=(
  manifest.v1.json
  genesis.v1.json
  genesis.v1.sha256
  validators.json
  bootnodes.json
  certificate.json
  profile.toml
  release-policy.toml
  topology/role-topology.toml
  topology/socket-matrix.toml
  topology/consensus-policy.toml
  topology/aoxcq-consensus.toml
)

resolve_aoxc_command() {
  if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
    AOXC_CMD=("${REPO_ROOT}/target/release/aoxc")
  else
    AOXC_CMD=(cargo run -q -p aoxcmd --)
  fi
}

run_aoxc() {
  local home="$1"
  shift
  if [[ -n "${home}" ]]; then
    AOXC_HOME="${home}" "${AOXC_CMD[@]}" "$@"
  else
    "${AOXC_CMD[@]}" "$@"
  fi
}

write_wrapper_script() {
  local wrapper_path="$1"
  if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
    cat > "${wrapper_path}" <<WRAPPER
#!/usr/bin/env bash
set -Eeuo pipefail
exec "${REPO_ROOT}/target/release/aoxc" "\$@"
WRAPPER
  else
    cat > "${wrapper_path}" <<WRAPPER
#!/usr/bin/env bash
set -Eeuo pipefail
cd "${REPO_ROOT}"
exec cargo run -q -p aoxcmd -- "\$@"
WRAPPER
  fi
  chmod +x "${wrapper_path}"
}

generate_secure_password() {
  python3 - <<'PYPASS'
import secrets
print(secrets.token_urlsafe(32))
PYPASS
}

extract_kv_field() {
  local file_path="$1"
  local field_name="$2"

  python3 - "$file_path" "$field_name" <<'PYKV'
import sys
from pathlib import Path

path = Path(sys.argv[1])
field_name = sys.argv[2]

try:
    lines = path.read_text(encoding="utf-8").splitlines()
except Exception:
    print("-")
    raise SystemExit(0)

pairs = {}
for raw in lines:
    line = raw.strip()
    if not line or ":" not in line:
        continue
    key, value = line.split(":", 1)
    pairs[key.strip()] = value.strip()

print(pairs.get(field_name, "-"))
PYKV
}

extract_account_id_field() {
  local file_path="$1"

  python3 - "$file_path" <<'PYADDR'
import sys
from pathlib import Path

path = Path(sys.argv[1])

try:
    lines = path.read_text(encoding="utf-8").splitlines()
except Exception:
    print("-")
    raise SystemExit(0)

pairs = {}
for raw in lines:
    line = raw.strip()
    if not line or ":" not in line:
        continue
    key, value = line.split(":", 1)
    pairs[key.strip()] = value.strip()

for candidate in (
    "validator_account_id",
    "account_id",
    "wallet_address",
    "address",
    "validator_account_id_legacy",
):
    value = pairs.get(candidate)
    if value:
        print(value)
        raise SystemExit(0)

print("-")
PYADDR
}

prepare_directories() {
  mkdir -p \
    "${TARGET_ROOT}/system/genesis" \
    "${TARGET_ROOT}/system/config" \
    "${TARGET_ROOT}/system/config/topology" \
    "${TARGET_ROOT}/system/config/metadata" \
    "${TARGET_ROOT}/system/audit" \
    "${TARGET_ROOT}/system/logs" \
    "${TARGET_ROOT}/system/scripts" \
    "${TARGET_ROOT}/nodes"
}

copy_environment_files() {
  cp "${SOURCE_ROOT}/manifest.v1.json" "${TARGET_ROOT}/system/genesis/manifest.json"
  cp "${SOURCE_ROOT}/genesis.v1.json" "${TARGET_ROOT}/system/genesis/genesis.json"
  cp "${SOURCE_ROOT}/genesis.v1.sha256" "${TARGET_ROOT}/system/genesis/genesis.sha256"

  cp "${SOURCE_ROOT}/validators.json" "${TARGET_ROOT}/system/config/validators.json"
  cp "${SOURCE_ROOT}/bootnodes.json" "${TARGET_ROOT}/system/config/bootnodes.json"
  cp "${SOURCE_ROOT}/certificate.json" "${TARGET_ROOT}/system/config/certificate.json"
  cp "${SOURCE_ROOT}/profile.toml" "${TARGET_ROOT}/system/config/profile.toml"
  cp "${SOURCE_ROOT}/release-policy.toml" "${TARGET_ROOT}/system/config/release-policy.toml"

  cp "${SOURCE_TOPOLOGY_ROOT}/role-topology.toml" "${TARGET_ROOT}/system/config/topology/role-topology.toml"
  cp "${SOURCE_TOPOLOGY_ROOT}/socket-matrix.toml" "${TARGET_ROOT}/system/config/topology/socket-matrix.toml"
  cp "${SOURCE_TOPOLOGY_ROOT}/consensus-policy.toml" "${TARGET_ROOT}/system/config/topology/consensus-policy.toml"
  cp "${SOURCE_TOPOLOGY_ROOT}/aoxcq-consensus.toml" "${TARGET_ROOT}/system/config/topology/aoxcq-consensus.toml"

  if [[ -f "${SOURCE_ROOT}/network-metadata.json" ]]; then
    cp "${SOURCE_ROOT}/network-metadata.json" "${TARGET_ROOT}/system/config/metadata/network-metadata.json"
  fi
}

node_rpc_port() { echo $((AOXC_Q_RPC_BASE_PORT + $1 - 1)); }
node_p2p_port() { echo $((AOXC_Q_P2P_BASE_PORT + $1 - 1)); }
node_metrics_port() { echo $((AOXC_Q_METRICS_BASE_PORT + $1 - 1)); }
node_admin_port() { echo $((AOXC_Q_ADMIN_BASE_PORT + $1 - 1)); }

validate_port_plan() {
  local max_rpc
  local max_p2p
  local max_metrics
  local max_admin
  max_rpc="$(node_rpc_port "${AOXC_Q_NODE_COUNT}")"
  max_p2p="$(node_p2p_port "${AOXC_Q_NODE_COUNT}")"
  max_metrics="$(node_metrics_port "${AOXC_Q_NODE_COUNT}")"
  max_admin="$(node_admin_port "${AOXC_Q_NODE_COUNT}")"

  ensure_port_in_range "${max_rpc}" "AOXC_Q_RPC_BASE_PORT + AOXC_Q_NODE_COUNT - 1"
  ensure_port_in_range "${max_p2p}" "AOXC_Q_P2P_BASE_PORT + AOXC_Q_NODE_COUNT - 1"
  ensure_port_in_range "${max_metrics}" "AOXC_Q_METRICS_BASE_PORT + AOXC_Q_NODE_COUNT - 1"
  ensure_port_in_range "${max_admin}" "AOXC_Q_ADMIN_BASE_PORT + AOXC_Q_NODE_COUNT - 1"
}

validate_genesis_checksum() {
  local genesis_file="${TARGET_ROOT}/system/genesis/genesis.json"
  local sha_file="${TARGET_ROOT}/system/genesis/genesis.sha256"
  local expected actual
  expected="$(awk '{print $1}' "${sha_file}")"
  actual="$(sha256sum "${genesis_file}" | awk '{print $1}')"
  [[ -n "${expected}" ]] || die "genesis.sha256 does not include a checksum value" 3
  [[ "${actual}" == "${expected}" ]] || die "genesis checksum mismatch (expected=${expected} actual=${actual})" 3
}

generate_seed_hex() {
  python3 - <<'PYSEED'
import secrets
print(secrets.token_hex(32))
PYSEED
}

write_node_ports_env() {
  local node_root="$1"
  local index="$2"

  cat > "${node_root}/ports.env" <<EOF
AOXC_NODE_RPC_PORT=$(node_rpc_port "${index}")
AOXC_NODE_P2P_PORT=$(node_p2p_port "${index}")
AOXC_NODE_METRICS_PORT=$(node_metrics_port "${index}")
AOXC_NODE_ADMIN_PORT=$(node_admin_port "${index}")
EOF

  chmod 600 "${node_root}/ports.env"
}

write_node_runtime_overlay() {
  local node_home="$1"
  local node_name="$2"
  local index="$3"

  cat > "${node_home}/config/runtime-overlay.toml" <<EOF
# Generated by AOXC rolling provisioner.
# This file captures deterministic node-local runtime metadata for audit
# and potential downstream runtime consumption.

[node]
name = "${node_name}"
environment = "${AOXC_Q_ENV}"
mode = "${AOXC_Q_MODE}"

[ports]
rpc = $(node_rpc_port "${index}")
p2p = $(node_p2p_port "${index}")
metrics = $(node_metrics_port "${index}")
admin = $(node_admin_port "${index}")

[paths]
manifest = "${node_home}/identity/manifest.json"
genesis = "${node_home}/identity/genesis.json"
validators = "${node_home}/config/validators.json"
bootnodes = "${node_home}/config/bootnodes.json"
certificate = "${node_home}/config/certificate.json"
profile = "${node_home}/config/profile.toml"
release_policy = "${node_home}/config/release-policy.toml"
role_topology = "${node_home}/config/topology/role-topology.toml"
socket_matrix = "${node_home}/config/topology/socket-matrix.toml"
consensus_policy = "${node_home}/config/topology/consensus-policy.toml"
aoxcq_consensus = "${node_home}/config/topology/aoxcq-consensus.toml"
network_metadata = "${node_home}/config/metadata/network-metadata.json"
EOF

  chmod 600 "${node_home}/config/runtime-overlay.toml"
}

write_node_static_config() {
  local out_file="$1"
  local node_name="$2"
  local node_home="$3"
  local index="$4"
  local validator_name="$5"
  local operator_name="$6"
  local validator_account_id="$7"
  local operator_account_id="$8"
  local seed_file="$9"

  cat > "${out_file}" <<EOF
# Generated by AOXC rolling provisioner.
node_name = "${node_name}"
environment = "${AOXC_Q_ENV}"
mode = "${AOXC_Q_MODE}"
index = ${index}
home = "${node_home}"
profile = "${AOXC_Q_PROFILE}"

[ports]
rpc = $(node_rpc_port "${index}")
p2p = $(node_p2p_port "${index}")
metrics = $(node_metrics_port "${index}")
admin = $(node_admin_port "${index}")

[identity]
validator_name = "${validator_name}"
operator_name = "${operator_name}"
validator_account_id = "${validator_account_id}"
operator_account_id = "${operator_account_id}"
seed_file = "${seed_file}"
genesis_file = "${node_home}/identity/genesis.json"
validators_file = "${node_home}/identity/validators.json"
bootnodes_file = "${node_home}/identity/bootnodes.json"
certificate_file = "${node_home}/identity/certificate.json"
EOF
}

write_topology_checksums() {
  sha256sum \
    "${TARGET_ROOT}/system/config/topology/role-topology.toml" \
    "${TARGET_ROOT}/system/config/topology/socket-matrix.toml" \
    "${TARGET_ROOT}/system/config/topology/consensus-policy.toml" \
    "${TARGET_ROOT}/system/config/topology/aoxcq-consensus.toml" \
    > "${TARGET_ROOT}/system/audit/topology.sha256"
}

wallet_count_total() {
  # per node: operator + validator wallets, plus one treasury system wallet
  echo $((AOXC_Q_NODE_COUNT * 2 + 1))
}

wallet_count_operator() {
  echo "${AOXC_Q_NODE_COUNT}"
}

wallet_count_validator() {
  echo "${AOXC_Q_NODE_COUNT}"
}

write_network_sizing_report() {
  local out_file="${TARGET_ROOT}/system/audit/network-sizing.txt"
  cat > "${out_file}" <<EOF
AOXC rolling network sizing
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
environment=${AOXC_Q_ENV}
profile=${AOXC_Q_PROFILE}
mode=${AOXC_Q_MODE}
real_testnet=${AOXC_Q_REAL_TESTNET}
node_count=${AOXC_Q_NODE_COUNT}
operator_wallet_count=$(wallet_count_operator)
validator_wallet_count=$(wallet_count_validator)
treasury_wallet_count=1
total_wallet_count=$(wallet_count_total)
EOF
}

write_node_identity_summary() {
  local node_root="$1"
  local node_name="$2"
  local operator_name="$3"
  local validator_name="$4"
  local operator_account_id="$5"
  local validator_account_id="$6"
  local operator_account_id_legacy="$7"
  local validator_account_id_legacy="$8"
  local operator_bundle_fingerprint="$9"
  local validator_bundle_fingerprint="${10}"
  local operator_consensus_public_key="${11}"
  local validator_consensus_public_key="${12}"
  local operator_transport_public_key="${13}"
  local validator_transport_public_key="${14}"

  cat > "${node_root}/identity-summary.env" <<EOF
NODE_NAME=${node_name}
OPERATOR_NAME=${operator_name}
VALIDATOR_NAME=${validator_name}
OPERATOR_ACCOUNT_ID=${operator_account_id}
VALIDATOR_ACCOUNT_ID=${validator_account_id}
OPERATOR_ACCOUNT_ID_LEGACY=${operator_account_id_legacy}
VALIDATOR_ACCOUNT_ID_LEGACY=${validator_account_id_legacy}
OPERATOR_BUNDLE_FINGERPRINT=${operator_bundle_fingerprint}
VALIDATOR_BUNDLE_FINGERPRINT=${validator_bundle_fingerprint}
OPERATOR_CONSENSUS_PUBLIC_KEY=${operator_consensus_public_key}
VALIDATOR_CONSENSUS_PUBLIC_KEY=${validator_consensus_public_key}
OPERATOR_TRANSPORT_PUBLIC_KEY=${operator_transport_public_key}
VALIDATOR_TRANSPORT_PUBLIC_KEY=${validator_transport_public_key}
PASSWORD_FILE=${node_root}/operator.password
EOF

  chmod 600 "${node_root}/identity-summary.env"

  cat > "${node_root}/identity-summary.tsv" <<EOF
field	value
node_name	${node_name}
operator_name	${operator_name}
validator_name	${validator_name}
operator_account_id	${operator_account_id}
validator_account_id	${validator_account_id}
operator_account_id_legacy	${operator_account_id_legacy}
validator_account_id_legacy	${validator_account_id_legacy}
operator_bundle_fingerprint	${operator_bundle_fingerprint}
validator_bundle_fingerprint	${validator_bundle_fingerprint}
operator_consensus_public_key	${operator_consensus_public_key}
validator_consensus_public_key	${validator_consensus_public_key}
operator_transport_public_key	${operator_transport_public_key}
validator_transport_public_key	${validator_transport_public_key}
password_file	${node_root}/operator.password
EOF

  chmod 600 "${node_root}/identity-summary.tsv"
}

render_node_runner() {
  local node_root="$1"
  local node_name="$2"
  local node_home="$3"
  local node_log="$4"
  local node_state_file="$5"
  local sleep_min_secs="$6"
  local sleep_max_secs="$7"
  local block_interval_secs="$8"

  cat > "${node_root}/run-node.sh" <<RUNNER
#!/usr/bin/env bash
set -Eeuo pipefail

NODE_NAME="${node_name}"
NODE_HOME="${node_home}"
ROUNDS="${AOXC_Q_ROUNDS}"
SLEEP_SECS="${AOXC_Q_SLEEP_SECS}"
SLEEP_MIN_SECS="${sleep_min_secs}"
SLEEP_MAX_SECS="${sleep_max_secs}"
BLOCK_INTERVAL_SECS="${block_interval_secs}"
WRAPPER="${TARGET_ROOT}/system/scripts/aoxc-wrapper.sh"
LOG_FILE="${node_log}"
STATE_FILE="${node_state_file}"
PORTS_FILE="${node_root}/ports.env"
HEARTBEAT_FILE="${node_root}/run/heartbeat.state"

mkdir -p "\$(dirname "\${LOG_FILE}")"

if [[ -f "\${PORTS_FILE}" ]]; then
  # shellcheck disable=SC1090
  source "\${PORTS_FILE}"
fi
cleanup() {
  ts_end="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
  printf '%s\tstatus=terminated\tnode=%s\n' "\${ts_end}" "\${NODE_NAME}" > "\${STATE_FILE}"
}
trap cleanup INT TERM

while true; do
  ts_start="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
  printf '%s\tstatus=running\tnode=%s\n' "\${ts_start}" "\${NODE_NAME}" > "\${HEARTBEAT_FILE}"
  if AOXC_HOME="\${NODE_HOME}" "\${WRAPPER}" node-run \
      --home "\${NODE_HOME}" \
      --rounds "\${ROUNDS}" \
      --interval-secs "\${BLOCK_INTERVAL_SECS}" \
      --tx-prefix "\${NODE_NAME^^}-TX" \
      --format json \
      --no-live-log \
      >>"\${LOG_FILE}" 2>&1; then
    ts_end="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
    printf '%s\tstatus=ok\tnode=%s\trpc=%s\tp2p=%s\tmetrics=%s\tadmin=%s\n' \
      "\${ts_end}" "\${NODE_NAME}" \
      "\${AOXC_NODE_RPC_PORT:-na}" "\${AOXC_NODE_P2P_PORT:-na}" \
      "\${AOXC_NODE_METRICS_PORT:-na}" "\${AOXC_NODE_ADMIN_PORT:-na}" \
      > "\${STATE_FILE}"
  else
    ts_end="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
    printf '%s\tstatus=error\tnode=%s\trpc=%s\tp2p=%s\tmetrics=%s\tadmin=%s\n' \
      "\${ts_end}" "\${NODE_NAME}" \
      "\${AOXC_NODE_RPC_PORT:-na}" "\${AOXC_NODE_P2P_PORT:-na}" \
      "\${AOXC_NODE_METRICS_PORT:-na}" "\${AOXC_NODE_ADMIN_PORT:-na}" \
      > "\${STATE_FILE}"
  fi

  ts_sleep="\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)"
  printf '%s\tstatus=idle\tnode=%s\n' "\${ts_sleep}" "\${NODE_NAME}" > "\${HEARTBEAT_FILE}"
  effective_sleep="\${SLEEP_SECS}"
  if (( SLEEP_MAX_SECS > SLEEP_MIN_SECS )); then
    span=\$((SLEEP_MAX_SECS - SLEEP_MIN_SECS + 1))
    effective_sleep=\$((SLEEP_MIN_SECS + RANDOM % span))
  else
    effective_sleep="\${SLEEP_MIN_SECS}"
  fi
  sleep "\${effective_sleep}"
done
RUNNER

  chmod +x "${node_root}/run-node.sh"
}

extract_json_field() {
  local json_payload="$1"
  local field_name="$2"
  JSON_PAYLOAD="${json_payload}" python3 - "$field_name" <<'PYJSON'
import json
import os
import sys

field_name = sys.argv[1]

try:
    payload = json.loads(os.environ.get("JSON_PAYLOAD", ""))
except Exception:
    print("-")
    raise SystemExit(0)

value = payload.get(field_name, "-")
if isinstance(value, bool):
    print("true" if value else "false")
elif value is None:
    print("-")
else:
    print(value)
PYJSON
}

fetch_balance_field() {
  local node_home="$1"
  local account_id="$2"
  local field_name="$3"
  local payload
  if ! payload="$(run_aoxc "${node_home}" balance-get --id "${account_id}" --format json 2>/dev/null)"; then
    echo "-"
    return 0
  fi
  extract_json_field "${payload}" "${field_name}"
}

write_cluster_monitor() {
  cat > "${TARGET_ROOT}/system/scripts/cluster-monitor.sh" <<MONITOR
#!/usr/bin/env bash
set -Eeuo pipefail

TARGET_ROOT="${TARGET_ROOT}"
NODE_COUNT="${AOXC_Q_NODE_COUNT}"
HEALTH_INTERVAL_SECS="${AOXC_Q_HEALTH_INTERVAL_SECS}"
LOG_FILE="${TARGET_ROOT}/system/logs/cluster-monitor.log"
STATE_FILE="${TARGET_ROOT}/system/logs/cluster-monitor.state"

mkdir -p "\$(dirname "\${LOG_FILE}")"

log_line() {
  local level="\$1"
  shift
  printf '%s\t%s\t%s\n' "\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)" "\${level}" "\$*" >> "\${LOG_FILE}"
}

restart_node() {
  local node_root="\$1"
  local node_name="\$2"
  nohup "\${node_root}/run-node.sh" > "\${node_root}/logs/supervisor.log" 2>&1 &
  echo "\$!" > "\${node_root}/node.pid"
  log_line info "restarted \${node_name} pid=\$(cat "\${node_root}/node.pid")"
}

while true; do
  local_running=0
  local_restarted=0
  for i in \$(seq 1 "\${NODE_COUNT}"); do
    node_name="node\$(printf '%02d' "\${i}")"
    node_root="\${TARGET_ROOT}/nodes/\${node_name}"
    pid_file="\${node_root}/node.pid"

    if [[ ! -f "\${node_root}/run-node.sh" ]]; then
      log_line warn "missing runner for \${node_name}"
      continue
    fi

    pid=""
    if [[ -f "\${pid_file}" ]]; then
      pid="\$(cat "\${pid_file}")"
    fi

    if [[ -n "\${pid}" ]] && kill -0 "\${pid}" 2>/dev/null; then
      local_running=\$((local_running + 1))
      continue
    fi

    restart_node "\${node_root}" "\${node_name}"
    local_restarted=\$((local_restarted + 1))
  done

  printf '%s\trunning=%s\trestarted=%s\tinterval=%s\n' \
    "\$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)" \
    "\${local_running}" \
    "\${local_restarted}" \
    "\${HEALTH_INTERVAL_SECS}" \
    > "\${STATE_FILE}"

  sleep "\${HEALTH_INTERVAL_SECS}"
done
MONITOR

  chmod +x "${TARGET_ROOT}/system/scripts/cluster-monitor.sh"
}

provision_testnet() {
  [[ -d "${SOURCE_ROOT}" ]] || die "Missing source environment: ${SOURCE_ROOT}" 3

  local required
  for required in "${required_source_files[@]}"; do
    [[ -f "${SOURCE_ROOT}/${required}" ]] || die "Missing required source file: ${SOURCE_ROOT}/${required}" 3
  done

  if [[ "${AOXC_Q_FORCE}" == "1" && -e "${TARGET_ROOT}" ]]; then
    chmod -R u+w "${TARGET_ROOT}" 2>/dev/null || true
    chattr -R -i "${TARGET_ROOT}" 2>/dev/null || true
    rm -rf "${TARGET_ROOT}"
  fi

  if [[ -e "${TARGET_ROOT}" ]]; then
    die "Target already exists: ${TARGET_ROOT} (use --force to recreate)" 4
  fi

  prepare_directories
  copy_environment_files
  validate_port_plan
  if [[ "${AOXC_Q_VALIDATE_GENESIS}" == "1" ]]; then
    validate_genesis_checksum
  fi
  write_wrapper_script "${TARGET_ROOT}/system/scripts/aoxc-wrapper.sh"
  write_cluster_monitor
  write_topology_checksums
  write_network_sizing_report

  local accounts_file="${TARGET_ROOT}/system/audit/prepared-accounts.tsv"
  local ports_file="${TARGET_ROOT}/system/audit/node-port-map.tsv"
  local seed_map_file="${TARGET_ROOT}/system/audit/node-seed-map.tsv"
  local balances_file="${TARGET_ROOT}/system/audit/wallet-balances.tsv"
  mkdir -p "${TARGET_ROOT}/system/config/nodes"

  printf 'node\tvalidator_name\toperator_name\tvalidator_account_id\toperator_account_id\tvalidator_account_id_legacy\toperator_account_id_legacy\tvalidator_bundle_fingerprint\toperator_bundle_fingerprint\tvalidator_consensus_public_key\toperator_consensus_public_key\tvalidator_transport_public_key\toperator_transport_public_key\tpassword_file\n' > "${accounts_file}"
  printf 'node\trpc_port\tp2p_port\tmetrics_port\tadmin_port\n' > "${ports_file}"
  printf 'node\tseed_file\tseed_sha256\n' > "${seed_map_file}"
  printf 'node\twallet_role\taccount_id\tknown\tbalance\tsource\n' > "${balances_file}"

  local i
  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local validator_name
    local operator_name
    local password
    local node_root
    local node_home
    local run_dir
    local log_dir
    local state_file
    local operator_create_json
    local validator_create_json
    local operator_account_id
    local validator_account_id
    local operator_account_id_legacy
    local validator_account_id_legacy
    local operator_bundle_fingerprint
    local validator_bundle_fingerprint
    local operator_consensus_public_key
    local validator_consensus_public_key
    local operator_transport_public_key
    local validator_transport_public_key
    local node_seed
    local node_seed_file
    local node_seed_sha
    local operator_known
    local validator_known
    local treasury_known
    local operator_balance
    local validator_balance
    local treasury_balance
    local operator_balance_source
    local validator_balance_source
    local treasury_balance_source

    node_name="node$(printf '%02d' "${i}")"
    validator_name="aoxcdev-val-$(printf '%02d' "${i}")"
    operator_name="aoxcdev-op-$(printf '%02d' "${i}")"
    password="$(generate_secure_password)"

    node_root="${TARGET_ROOT}/nodes/${node_name}"
    node_home="${node_root}/home"
    run_dir="${node_root}/run"
    log_dir="${node_root}/logs"
    state_file="${run_dir}/loop.state"

    mkdir -p \
      "${node_home}/identity" \
      "${node_home}/config" \
      "${node_home}/config/topology" \
      "${node_home}/config/metadata" \
      "${node_home}/runtime" \
      "${node_home}/audit" \
      "${run_dir}" \
      "${log_dir}"

    cp "${TARGET_ROOT}/system/genesis/manifest.json" "${node_home}/identity/manifest.json"
    cp "${TARGET_ROOT}/system/genesis/genesis.json" "${node_home}/identity/genesis.json"
    cp "${TARGET_ROOT}/system/config/validators.json" "${node_home}/identity/validators.json"
    cp "${TARGET_ROOT}/system/config/bootnodes.json" "${node_home}/identity/bootnodes.json"
    cp "${TARGET_ROOT}/system/config/certificate.json" "${node_home}/identity/certificate.json"

    cp "${TARGET_ROOT}/system/config/profile.toml" "${node_home}/config/profile.toml"
    cp "${TARGET_ROOT}/system/config/release-policy.toml" "${node_home}/config/release-policy.toml"
    cp "${TARGET_ROOT}/system/config/validators.json" "${node_home}/config/validators.json"
    cp "${TARGET_ROOT}/system/config/bootnodes.json" "${node_home}/config/bootnodes.json"
    cp "${TARGET_ROOT}/system/config/certificate.json" "${node_home}/config/certificate.json"

    cp "${TARGET_ROOT}/system/config/topology/role-topology.toml" "${node_home}/config/topology/role-topology.toml"
    cp "${TARGET_ROOT}/system/config/topology/socket-matrix.toml" "${node_home}/config/topology/socket-matrix.toml"
    cp "${TARGET_ROOT}/system/config/topology/consensus-policy.toml" "${node_home}/config/topology/consensus-policy.toml"
    cp "${TARGET_ROOT}/system/config/topology/aoxcq-consensus.toml" "${node_home}/config/topology/aoxcq-consensus.toml"

    if [[ -f "${TARGET_ROOT}/system/config/metadata/network-metadata.json" ]]; then
      cp "${TARGET_ROOT}/system/config/metadata/network-metadata.json" "${node_home}/config/metadata/network-metadata.json"
    fi

    printf '%s\n' "${password}" > "${node_root}/operator.password"
    chmod 600 "${node_root}/operator.password"
    node_seed="$(generate_seed_hex)"
    node_seed_file="${node_home}/identity/node-seed.hex"
    printf '%s\n' "${node_seed}" > "${node_seed_file}"
    chmod 600 "${node_seed_file}"
    node_seed_sha="$(sha256sum "${node_seed_file}" | awk '{print $1}')"

    write_node_ports_env "${node_root}" "${i}"
    write_node_runtime_overlay "${node_home}" "${node_name}" "${i}"

    run_aoxc "${node_home}" config-init --profile "${AOXC_Q_PROFILE}" --json-logs > "${run_dir}/config-init.json"

    run_aoxc "${node_home}" address-create \
      --name "${operator_name}" \
      --profile "${AOXC_Q_PROFILE}" \
      --password "${password}" \
      > "${run_dir}/address-create-operator.txt"

    run_aoxc "${node_home}" address-create \
      --name "${validator_name}" \
      --profile "${AOXC_Q_PROFILE}" \
      --password "${password}" \
      > "${run_dir}/address-create-validator.txt"

    run_aoxc "${node_home}" key-bootstrap \
      --force \
      --profile "${AOXC_Q_PROFILE}" \
      --name "${validator_name}" \
      --password "${password}" \
      > "${run_dir}/key-bootstrap.txt"

    run_aoxc "${node_home}" keys-verify \
      --password "${password}" \
      > "${run_dir}/keys-verify.txt"

    run_aoxc "" node-bootstrap --home "${node_home}" > "${run_dir}/node-bootstrap.txt"

    operator_create_json="${run_dir}/address-create-operator.txt"
    validator_create_json="${run_dir}/address-create-validator.txt"

    operator_account_id="$(extract_account_id_field "${operator_create_json}")"
    validator_account_id="$(extract_account_id_field "${validator_create_json}")"
    [[ "${operator_account_id}" != "-" ]] || die "Failed to resolve operator account id for ${node_name}" 6
    [[ "${validator_account_id}" != "-" ]] || die "Failed to resolve validator account id for ${node_name}" 6

    operator_account_id_legacy="$(extract_kv_field "${operator_create_json}" "validator_account_id_legacy")"
    validator_account_id_legacy="$(extract_kv_field "${validator_create_json}" "validator_account_id_legacy")"

    operator_bundle_fingerprint="$(extract_kv_field "${operator_create_json}" "bundle_fingerprint")"
    validator_bundle_fingerprint="$(extract_kv_field "${validator_create_json}" "bundle_fingerprint")"

    operator_consensus_public_key="$(extract_kv_field "${operator_create_json}" "consensus_public_key")"
    validator_consensus_public_key="$(extract_kv_field "${validator_create_json}" "consensus_public_key")"

    operator_transport_public_key="$(extract_kv_field "${operator_create_json}" "transport_public_key")"
    validator_transport_public_key="$(extract_kv_field "${validator_create_json}" "transport_public_key")"

    run_aoxc "${node_home}" economy-init --format json > "${run_dir}/economy-init.json"
    run_aoxc "${node_home}" treasury-transfer \
      --to "${operator_account_id}" \
      --amount "${AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE}" \
      --format json \
      > "${run_dir}/treasury-transfer-operator.json"
    run_aoxc "${node_home}" treasury-transfer \
      --to "${validator_account_id}" \
      --amount "${AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE}" \
      --format json \
      > "${run_dir}/treasury-transfer-validator.json"

    operator_known="$(fetch_balance_field "${node_home}" "${operator_account_id}" "known")"
    operator_balance="$(fetch_balance_field "${node_home}" "${operator_account_id}" "balance")"
    operator_balance_source="$(fetch_balance_field "${node_home}" "${operator_account_id}" "source")"

    validator_known="$(fetch_balance_field "${node_home}" "${validator_account_id}" "known")"
    validator_balance="$(fetch_balance_field "${node_home}" "${validator_account_id}" "balance")"
    validator_balance_source="$(fetch_balance_field "${node_home}" "${validator_account_id}" "source")"

    treasury_known="$(fetch_balance_field "${node_home}" "treasury" "known")"
    treasury_balance="$(fetch_balance_field "${node_home}" "treasury" "balance")"
    treasury_balance_source="$(fetch_balance_field "${node_home}" "treasury" "source")"

    write_node_identity_summary \
      "${node_root}" \
      "${node_name}" \
      "${operator_name}" \
      "${validator_name}" \
      "${operator_account_id}" \
      "${validator_account_id}" \
      "${operator_account_id_legacy}" \
      "${validator_account_id_legacy}" \
      "${operator_bundle_fingerprint}" \
      "${validator_bundle_fingerprint}" \
      "${operator_consensus_public_key}" \
      "${validator_consensus_public_key}" \
      "${operator_transport_public_key}" \
      "${validator_transport_public_key}"

    write_node_static_config \
      "${node_root}/node-config.toml" \
      "${node_name}" \
      "${node_home}" \
      "${i}" \
      "${validator_name}" \
      "${operator_name}" \
      "${validator_account_id}" \
      "${operator_account_id}" \
      "${node_seed_file}"
    cp "${node_root}/node-config.toml" "${TARGET_ROOT}/system/config/nodes/${node_name}.toml"
    chmod 600 "${node_root}/node-config.toml" "${TARGET_ROOT}/system/config/nodes/${node_name}.toml"

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "${node_name}" "${validator_name}" "${operator_name}" \
      "${validator_account_id}" "${operator_account_id}" \
      "${validator_account_id_legacy}" "${operator_account_id_legacy}" \
      "${validator_bundle_fingerprint}" "${operator_bundle_fingerprint}" \
      "${validator_consensus_public_key}" "${operator_consensus_public_key}" \
      "${validator_transport_public_key}" "${operator_transport_public_key}" \
      "${node_root}/operator.password" \
      >> "${accounts_file}"

    printf '%s\t%s\t%s\t%s\t%s\n' \
      "${node_name}" \
      "$(node_rpc_port "${i}")" \
      "$(node_p2p_port "${i}")" \
      "$(node_metrics_port "${i}")" \
      "$(node_admin_port "${i}")" \
      >> "${ports_file}"
    printf '%s\t%s\t%s\n' \
      "${node_name}" \
      "${node_seed_file}" \
      "${node_seed_sha}" \
      >> "${seed_map_file}"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
      "${node_name}" \
      "operator" \
      "${operator_account_id}" \
      "${operator_known}" \
      "${operator_balance}" \
      "${operator_balance_source}" \
      >> "${balances_file}"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
      "${node_name}" \
      "validator" \
      "${validator_account_id}" \
      "${validator_known}" \
      "${validator_balance}" \
      "${validator_balance_source}" \
      >> "${balances_file}"
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
      "${node_name}" \
      "treasury" \
      "treasury" \
      "${treasury_known}" \
      "${treasury_balance}" \
      "${treasury_balance_source}" \
      >> "${balances_file}"

    render_node_runner \
      "${node_root}" \
      "${node_name}" \
      "${node_home}" \
      "${log_dir}/node-run.log" \
      "${state_file}" \
      "${AOXC_Q_SLEEP_MIN_SECS}" \
      "${AOXC_Q_SLEEP_MAX_SECS}" \
      "${AOXC_Q_BLOCK_INTERVAL_SECS}"
  done

  chmod -R go-rwx "${TARGET_ROOT}" || true
  chmod -R u+rwX "${TARGET_ROOT}" || true

  cat > "${TARGET_ROOT}/system/audit/provision-report.txt" <<REPORT
AOXC rolling local bootstrap report
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
repo_root=${REPO_ROOT}
environment=${AOXC_Q_ENV}
profile=${AOXC_Q_PROFILE}
mode=${AOXC_Q_MODE}
node_count=${AOXC_Q_NODE_COUNT}
real_testnet=${AOXC_Q_REAL_TESTNET}
operator_wallet_count=$(wallet_count_operator)
validator_wallet_count=$(wallet_count_validator)
treasury_wallet_count=1
total_wallet_count=$(wallet_count_total)
rounds=${AOXC_Q_ROUNDS}
sleep_secs=${AOXC_Q_SLEEP_SECS}
sleep_min_secs=${AOXC_Q_SLEEP_MIN_SECS}
sleep_max_secs=${AOXC_Q_SLEEP_MAX_SECS}
health_interval_secs=${AOXC_Q_HEALTH_INTERVAL_SECS}
block_interval_secs=${AOXC_Q_BLOCK_INTERVAL_SECS}
operator_bootstrap_balance=${AOXC_Q_OPERATOR_BOOTSTRAP_BALANCE}
validator_bootstrap_balance=${AOXC_Q_VALIDATOR_BOOTSTRAP_BALANCE}
rpc_base_port=${AOXC_Q_RPC_BASE_PORT}
p2p_base_port=${AOXC_Q_P2P_BASE_PORT}
metrics_base_port=${AOXC_Q_METRICS_BASE_PORT}
admin_base_port=${AOXC_Q_ADMIN_BASE_PORT}
root=${TARGET_ROOT}
manifest_file=${TARGET_ROOT}/system/genesis/manifest.json
genesis_file=${TARGET_ROOT}/system/genesis/genesis.json
topology_role_file=${TARGET_ROOT}/system/config/topology/role-topology.toml
topology_socket_matrix_file=${TARGET_ROOT}/system/config/topology/socket-matrix.toml
topology_consensus_policy_file=${TARGET_ROOT}/system/config/topology/consensus-policy.toml
topology_aoxcq_consensus_file=${TARGET_ROOT}/system/config/topology/aoxcq-consensus.toml
node_config_dir=${TARGET_ROOT}/system/config/nodes
network_metadata_present=$([[ -f "${TARGET_ROOT}/system/config/metadata/network-metadata.json" ]] && echo yes || echo no)
accounts_file=${TARGET_ROOT}/system/audit/prepared-accounts.tsv
ports_file=${TARGET_ROOT}/system/audit/node-port-map.tsv
seed_map_file=${TARGET_ROOT}/system/audit/node-seed-map.tsv
wallet_balances_file=${TARGET_ROOT}/system/audit/wallet-balances.tsv
topology_checksums=${TARGET_ROOT}/system/audit/topology.sha256
network_sizing_file=${TARGET_ROOT}/system/audit/network-sizing.txt
genesis_checksum_validation=${AOXC_Q_VALIDATE_GENESIS}
REPORT

  log_info "Provisioning complete: ${TARGET_ROOT}"
}

start_testnet() {
  [[ -d "${TARGET_ROOT}/nodes" ]] || die "Target root is not provisioned: ${TARGET_ROOT}" 5

  local i
  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local node_root
    local pid_file
    local pid

    node_name="node$(printf '%02d' "${i}")"
    node_root="${TARGET_ROOT}/nodes/${node_name}"
    pid_file="${node_root}/node.pid"

    if [[ -f "${pid_file}" ]]; then
      pid="$(cat "${pid_file}")"
      if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
        log_info "${node_name} already running pid=${pid}"
        continue
      fi
      rm -f "${pid_file}"
    fi

    nohup "${node_root}/run-node.sh" > "${node_root}/logs/supervisor.log" 2>&1 &
    echo "$!" > "${pid_file}"
    log_info "started ${node_name} pid=$(cat "${pid_file}")"
  done

  local monitor_pid_file="${TARGET_ROOT}/system/logs/cluster-monitor.pid"
  local monitor_pid=""
  if [[ -f "${monitor_pid_file}" ]]; then
    monitor_pid="$(cat "${monitor_pid_file}")"
  fi
  if [[ -n "${monitor_pid}" ]] && kill -0 "${monitor_pid}" 2>/dev/null; then
    log_info "cluster monitor already running pid=${monitor_pid}"
  else
    nohup "${TARGET_ROOT}/system/scripts/cluster-monitor.sh" > "${TARGET_ROOT}/system/logs/cluster-monitor.nohup.log" 2>&1 &
    echo "$!" > "${monitor_pid_file}"
    log_info "cluster monitor started pid=$(cat "${monitor_pid_file}")"
  fi
}

stop_testnet() {
  [[ -d "${TARGET_ROOT}/nodes" ]] || die "Target root is not provisioned: ${TARGET_ROOT}" 5

  local i
  local monitor_pid_file="${TARGET_ROOT}/system/logs/cluster-monitor.pid"
  local monitor_pid=""
  if [[ -f "${monitor_pid_file}" ]]; then
    monitor_pid="$(cat "${monitor_pid_file}")"
    if [[ -n "${monitor_pid}" ]] && kill -0 "${monitor_pid}" 2>/dev/null; then
      kill "${monitor_pid}" || true
      sleep 1
      if kill -0 "${monitor_pid}" 2>/dev/null; then
        kill -9 "${monitor_pid}" || true
      fi
      log_info "stopped cluster monitor pid=${monitor_pid}"
    else
      log_warn "cluster monitor pid file stale (${monitor_pid})"
    fi
    rm -f "${monitor_pid_file}"
  fi

  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local node_root
    local pid_file
    local pid

    node_name="node$(printf '%02d' "${i}")"
    node_root="${TARGET_ROOT}/nodes/${node_name}"
    pid_file="${node_root}/node.pid"

    [[ -f "${pid_file}" ]] || { log_info "${node_name} not running"; continue; }

    pid="$(cat "${pid_file}")"
    if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" || true
      sleep 1
      if kill -0 "${pid}" 2>/dev/null; then
        kill -9 "${pid}" || true
      fi
      log_info "stopped ${node_name} pid=${pid}"
    else
      log_warn "${node_name} pid file stale (${pid})"
    fi
    rm -f "${pid_file}"
  done
}

status_testnet() {
  [[ -d "${TARGET_ROOT}/nodes" ]] || die "Target root is not provisioned: ${TARGET_ROOT}" 5
  local monitor_pid_file="${TARGET_ROOT}/system/logs/cluster-monitor.pid"
  local monitor_state="stopped"
  if [[ -f "${monitor_pid_file}" ]]; then
    local monitor_pid
    monitor_pid="$(cat "${monitor_pid_file}")"
    if [[ -n "${monitor_pid}" ]] && kill -0 "${monitor_pid}" 2>/dev/null; then
      monitor_state="running(pid=${monitor_pid})"
    else
      monitor_state="stale-pid(${monitor_pid})"
    fi
  fi
  printf 'cluster_monitor\t%s\n' "${monitor_state}"
  printf 'node\tprocess\tpid\trpc\tp2p\tmetrics\tadmin\theight\tupdated_at\n'

  local i
  for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
    local node_name
    local node_root
    local pid_file
    local state_json
    local process_state
    local pid_text
    local height
    local updated_at

    node_name="node$(printf '%02d' "${i}")"
    node_root="${TARGET_ROOT}/nodes/${node_name}"
    pid_file="${node_root}/node.pid"
    process_state="stopped"
    pid_text="-"

    if [[ -f "${pid_file}" ]]; then
      pid_text="$(cat "${pid_file}")"
      if [[ -n "${pid_text}" ]] && kill -0 "${pid_text}" 2>/dev/null; then
        process_state="running"
      else
        process_state="stale-pid"
      fi
    fi

    height="-"
    updated_at="-"
    if state_json="$(run_aoxc "${node_root}/home" chain-status --format json 2>/dev/null)"; then
      height="$(printf '%s' "${state_json}" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("current_height","-"))' 2>/dev/null || echo '-')"
      updated_at="$(printf '%s' "${state_json}" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("updated_at","-"))' 2>/dev/null || echo '-')"
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "${node_name}" \
      "${process_state}" \
      "${pid_text}" \
      "$(node_rpc_port "${i}")" \
      "$(node_p2p_port "${i}")" \
      "$(node_metrics_port "${i}")" \
      "$(node_admin_port "${i}")" \
      "${height}" \
      "${updated_at}"
  done
}

main() {
  resolve_aoxc_command

  case "${AOXC_Q_ACTION}" in
    up)
      provision_testnet
      start_testnet
      ;;
    provision)
      provision_testnet
      ;;
    start)
      start_testnet
      ;;
    stop)
      stop_testnet
      ;;
    restart)
      stop_testnet
      start_testnet
      ;;
    status)
      status_testnet
      ;;
  esac

  if [[ "${AOXC_Q_ACTION}" == "up" || "${AOXC_Q_ACTION}" == "provision" ]]; then
    log_info "accounts: ${TARGET_ROOT}/system/audit/prepared-accounts.tsv"
    log_info "ports: ${TARGET_ROOT}/system/audit/node-port-map.tsv"
    log_info "wallet balances: ${TARGET_ROOT}/system/audit/wallet-balances.tsv"
    log_info "provision report: ${TARGET_ROOT}/system/audit/provision-report.txt"
    log_info "topology checksums: ${TARGET_ROOT}/system/audit/topology.sha256"
    log_info "network sizing: ${TARGET_ROOT}/system/audit/network-sizing.txt"
    log_info "capacity summary: nodes=${AOXC_Q_NODE_COUNT} operator_wallets=$(wallet_count_operator) validator_wallets=$(wallet_count_validator) treasury_wallets=1 total_wallets=$(wallet_count_total)"
    log_info "control: $(basename "$0") --action start|stop|status --home ${AOXC_Q_HOME} --env ${AOXC_Q_ENV} --nodes ${AOXC_Q_NODE_COUNT}"
  fi
}

main "$@"
