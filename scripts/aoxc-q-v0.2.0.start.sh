#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AOXC_Q_HOME="${AOXC_Q_HOME:-/mnt/xdbx/aoxc}"
AOXC_Q_ENV="${AOXC_Q_ENV:-testnet}"
AOXC_Q_PROFILE="${AOXC_Q_PROFILE:-testnet}"
AOXC_Q_NODE_COUNT="${AOXC_Q_NODE_COUNT:-7}"
AOXC_Q_ROUNDS="${AOXC_Q_ROUNDS:-200}"
AOXC_Q_START="${AOXC_Q_START:-1}"
AOXC_Q_SLEEP_MS="${AOXC_Q_SLEEP_MS:-250}"
AOXC_Q_FORCE="${AOXC_Q_FORCE:-0}"

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Prepare AOXC-Q v0.2.0 style seven-node local testnet layout without Docker/Podman,
bootstrap all nodes, create per-node validator/operator accounts, and optionally
start persistent local node-run loops.

Options:
  --home <path>         Root path (default: ${AOXC_Q_HOME})
  --env <name>          configs/environments/<name> source (default: ${AOXC_Q_ENV})
  --profile <name>      AOXC profile for config/key bootstrap (default: ${AOXC_Q_PROFILE})
  --rounds <n>          rounds per node-run invocation (default: ${AOXC_Q_ROUNDS})
  --sleep-ms <n>        sleep between rounds in ms (default: ${AOXC_Q_SLEEP_MS})
  --no-start            bootstrap only; do not start persistent loops
  --force               remove existing target root first
  -h, --help            show this help

Environment:
  AOXC_Q_HOME, AOXC_Q_ENV, AOXC_Q_PROFILE, AOXC_Q_NODE_COUNT,
  AOXC_Q_ROUNDS, AOXC_Q_START, AOXC_Q_SLEEP_MS, AOXC_Q_FORCE
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --home) AOXC_Q_HOME="$2"; shift 2 ;;
    --env) AOXC_Q_ENV="$2"; shift 2 ;;
    --profile) AOXC_Q_PROFILE="$2"; shift 2 ;;
    --rounds) AOXC_Q_ROUNDS="$2"; shift 2 ;;
    --sleep-ms) AOXC_Q_SLEEP_MS="$2"; shift 2 ;;
    --no-start) AOXC_Q_START=0; shift ;;
    --force) AOXC_Q_FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

require_uint() {
  local value="$1"
  local name="$2"
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    echo "${name} must be an unsigned integer (got: ${value})" >&2
    exit 2
  fi
}

require_uint "${AOXC_Q_NODE_COUNT}" "AOXC_Q_NODE_COUNT"
require_uint "${AOXC_Q_ROUNDS}" "AOXC_Q_ROUNDS"
require_uint "${AOXC_Q_SLEEP_MS}" "AOXC_Q_SLEEP_MS"

if (( AOXC_Q_NODE_COUNT < 7 )); then
  echo "AOXC_Q_NODE_COUNT must be >= 7 for this production-like topology script." >&2
  exit 2
fi

SOURCE_ROOT="${REPO_ROOT}/configs/environments/${AOXC_Q_ENV}"
if [[ ! -d "${SOURCE_ROOT}" ]]; then
  echo "Missing source environment: ${SOURCE_ROOT}" >&2
  exit 3
fi

for required in manifest.v1.json genesis.v1.json genesis.v1.sha256 validators.json bootnodes.json certificate.json profile.toml release-policy.toml; do
  if [[ ! -f "${SOURCE_ROOT}/${required}" ]]; then
    echo "Missing required source file: ${SOURCE_ROOT}/${required}" >&2
    exit 3
  fi
done

TARGET_ROOT="${AOXC_Q_HOME%/}/aoxc-q-v0.2.0-${AOXC_Q_ENV}-${AOXC_Q_NODE_COUNT}n"
if [[ "${AOXC_Q_FORCE}" == "1" && -e "${TARGET_ROOT}" ]]; then
  chmod -R u+w "${TARGET_ROOT}" 2>/dev/null || true
  chattr -R -i "${TARGET_ROOT}" 2>/dev/null || true
  rm -rf "${TARGET_ROOT}"
fi
if [[ -e "${TARGET_ROOT}" ]]; then
  echo "Target already exists: ${TARGET_ROOT}" >&2
  echo "Use --force to recreate." >&2
  exit 4
fi

AOXC_BIN_CMD=( )
if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
  AOXC_BIN_CMD=("${REPO_ROOT}/target/release/aoxc")
else
  AOXC_BIN_CMD=(cargo run -q -p aoxcmd --)
fi

mkdir -p \
  "${TARGET_ROOT}/system/genesis" \
  "${TARGET_ROOT}/system/config" \
  "${TARGET_ROOT}/system/audit" \
  "${TARGET_ROOT}/system/logs" \
  "${TARGET_ROOT}/system/scripts" \
  "${TARGET_ROOT}/nodes"

cp "${SOURCE_ROOT}/manifest.v1.json" "${TARGET_ROOT}/system/genesis/manifest.json"
cp "${SOURCE_ROOT}/genesis.v1.json" "${TARGET_ROOT}/system/genesis/genesis.json"
cp "${SOURCE_ROOT}/genesis.v1.sha256" "${TARGET_ROOT}/system/genesis/genesis.sha256"
cp "${SOURCE_ROOT}/validators.json" "${TARGET_ROOT}/system/config/validators.json"
cp "${SOURCE_ROOT}/bootnodes.json" "${TARGET_ROOT}/system/config/bootnodes.json"
cp "${SOURCE_ROOT}/certificate.json" "${TARGET_ROOT}/system/config/certificate.json"
cp "${SOURCE_ROOT}/profile.toml" "${TARGET_ROOT}/system/config/profile.toml"
cp "${SOURCE_ROOT}/release-policy.toml" "${TARGET_ROOT}/system/config/release-policy.toml"

ACCOUNTS_FILE="${TARGET_ROOT}/system/audit/prepared-accounts.tsv"
printf 'node\tvalidator_name\toperator_name\n' > "${ACCOUNTS_FILE}"

for i in $(seq 1 "${AOXC_Q_NODE_COUNT}"); do
  node_name="node$(printf '%02d' "${i}")"
  validator_name="aoxcq-val-$(printf '%02d' "${i}")"
  operator_name="aoxcq-op-$(printf '%02d' "${i}")"
  password="AOXC-Q-${AOXC_Q_ENV^^}-${node_name^^}-CHANGE-ME"

  node_root="${TARGET_ROOT}/nodes/${node_name}"
  node_home="${node_root}/home"
  run_dir="${node_root}/run"
  log_dir="${node_root}/logs"

  mkdir -p "${node_home}/identity" "${node_home}/config" "${node_home}/runtime" "${node_home}/audit" "${run_dir}" "${log_dir}"

  cp "${TARGET_ROOT}/system/genesis/genesis.json" "${node_home}/identity/genesis.json"
  cp "${TARGET_ROOT}/system/config/profile.toml" "${node_home}/config/profile.toml"
  cp "${TARGET_ROOT}/system/config/validators.json" "${node_home}/config/validators.json"
  cp "${TARGET_ROOT}/system/config/bootnodes.json" "${node_home}/config/bootnodes.json"

  printf '%s\n' "${password}" > "${node_root}/operator.password"
  chmod 600 "${node_root}/operator.password" || true

  AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" config-init --profile "${AOXC_Q_PROFILE}" --json-logs > "${run_dir}/config-init.json"
  AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" address-create --name "${operator_name}" --profile "${AOXC_Q_PROFILE}" --password "${password}" > "${run_dir}/address-create-operator.json"
  AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" address-create --name "${validator_name}" --profile "${AOXC_Q_PROFILE}" --password "${password}" > "${run_dir}/address-create-validator.json"
  AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" key-bootstrap --profile "${AOXC_Q_PROFILE}" --name "${validator_name}" --password "${password}" > "${run_dir}/key-bootstrap.json"
  AOXC_HOME="${node_home}" "${AOXC_BIN_CMD[@]}" keys-verify --password "${password}" > "${run_dir}/keys-verify.json"
  "${AOXC_BIN_CMD[@]}" node-bootstrap --home "${node_home}" > "${run_dir}/node-bootstrap.json"

  printf '%s\t%s\t%s\n' "${node_name}" "${validator_name}" "${operator_name}" >> "${ACCOUNTS_FILE}"

  cat > "${node_root}/run-node.sh" <<RUNNER
#!/usr/bin/env bash
set -Eeuo pipefail
AOXC_HOME="${node_home}"
AOXC_BIN="${AOXC_BIN_CMD[0]}"
while true; do
  "\${AOXC_BIN}" node-run --home "\${AOXC_HOME}" --rounds "${AOXC_Q_ROUNDS}" --sleep-ms "${AOXC_Q_SLEEP_MS}" --tx-prefix "${node_name^^}-TX" >> "${log_dir}/node-run.log" 2>&1 || true
  sleep 1
done
RUNNER
  chmod +x "${node_root}/run-node.sh"
done

cat > "${TARGET_ROOT}/system/scripts/start-all.sh" <<STARTALL
#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="${TARGET_ROOT}"
for i in \
$(seq 1 "${AOXC_Q_NODE_COUNT}" | sed 's/^/  /')
; do
  node_name="node\$(printf '%02d' "\${i}")"
  node_root="\${ROOT}/nodes/\${node_name}"
  if [[ -f "\${node_root}/node.pid" ]] && kill -0 "\$(cat "\${node_root}/node.pid")" 2>/dev/null; then
    echo "[skip] \${node_name} already running"
    continue
  fi
  nohup "\${node_root}/run-node.sh" >/dev/null 2>&1 &
  echo \$! > "\${node_root}/node.pid"
  echo "[ok] started \${node_name} pid=\$(cat "\${node_root}/node.pid")"
done
STARTALL
chmod +x "${TARGET_ROOT}/system/scripts/start-all.sh"

cat > "${TARGET_ROOT}/system/scripts/stop-all.sh" <<STOPALL
#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="${TARGET_ROOT}"
for i in \
$(seq 1 "${AOXC_Q_NODE_COUNT}" | sed 's/^/  /')
; do
  node_name="node\$(printf '%02d' "\${i}")"
  node_root="\${ROOT}/nodes/\${node_name}"
  if [[ -f "\${node_root}/node.pid" ]]; then
    pid="\$(cat "\${node_root}/node.pid")"
    if kill -0 "\${pid}" 2>/dev/null; then
      kill "\${pid}" || true
      echo "[ok] stopped \${node_name} pid=\${pid}"
    fi
    rm -f "\${node_root}/node.pid"
  fi
done
STOPALL
chmod +x "${TARGET_ROOT}/system/scripts/stop-all.sh"

if [[ "${AOXC_Q_START}" == "1" ]]; then
  "${TARGET_ROOT}/system/scripts/start-all.sh"
fi

cat > "${TARGET_ROOT}/system/audit/provision-report.txt" <<REPORT
AOXC-Q v0.2.0 local testnet bootstrap report
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
repo_root=${REPO_ROOT}
environment=${AOXC_Q_ENV}
profile=${AOXC_Q_PROFILE}
node_count=${AOXC_Q_NODE_COUNT}
rounds=${AOXC_Q_ROUNDS}
sleep_ms=${AOXC_Q_SLEEP_MS}
root=${TARGET_ROOT}
started=${AOXC_Q_START}
REPORT

chmod -R go-rwx "${TARGET_ROOT}" || true
chmod -R u+rwX "${TARGET_ROOT}" || true

echo "AOXC-Q local ${AOXC_Q_NODE_COUNT}-node layout prepared at: ${TARGET_ROOT}"
echo "Accounts manifest: ${ACCOUNTS_FILE}"
echo "Start script: ${TARGET_ROOT}/system/scripts/start-all.sh"
echo "Stop script:  ${TARGET_ROOT}/system/scripts/stop-all.sh"
