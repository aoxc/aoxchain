#!/usr/bin/env bash
# AOXC MIT License
# Generates and optionally executes a production-oriented multi-node testnet bootstrap plan.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

AOXC_BIN="${AOXC_BIN:-${REPO_ROOT}/target/release/aoxc}"
ROOT_DIR="${AOXC_TESTNET_ROOT:-${HOME}/.aoxc-testnet-prod}"
PROFILE="testnet"
VALIDATORS="${AOXC_TESTNET_VALIDATORS:-7}"
BOOTNODES="${AOXC_TESTNET_BOOTNODES:-3}"
RPC_NODES="${AOXC_TESTNET_RPC_NODES:-2}"
ARCHIVE_NODES="${AOXC_TESTNET_ARCHIVE_NODES:-2}"
SENTRY_PER_VALIDATOR="${AOXC_TESTNET_SENTRY_PER_VALIDATOR:-2}"
PASSWORD="${AOXC_TESTNET_PASSWORD:-}"
BIND_HOST="${AOXC_TESTNET_BIND_HOST:-0.0.0.0}"
EXECUTE=0
FORCE=0

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Create a production-grade testnet topology plan and per-node bootstrap/start scripts.
By default this creates artifacts only (dry-run plan generation).
Use --execute to run bootstrap/start commands locally for all generated nodes.

Options:
  --root <path>                 Testnet root directory (default: ${ROOT_DIR})
  --validators <n>              Validator count, minimum 3 for testnet (default: ${VALIDATORS})
  --bootnodes <n>               Bootnode count, minimum 2 for testnet (default: ${BOOTNODES})
  --rpc <n>                     RPC node count (default: ${RPC_NODES})
  --archive <n>                 Archive node count (default: ${ARCHIVE_NODES})
  --sentry-per-validator <n>    Sentry nodes per validator (default: ${SENTRY_PER_VALIDATOR})
  --password <value>            Password used by production-bootstrap
  --bind-host <host>            Bind host passed to production-bootstrap (default: ${BIND_HOST})
  --aoxc-bin <path>             AOXC binary path (default: ${AOXC_BIN})
  --execute                     Execute generated commands for all nodes
  --force                       Remove existing --root before generating
  -h, --help                    Show this help
USAGE
}

require_uint() {
  local value="$1"
  local label="$2"
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    echo "${label} must be an unsigned integer: ${value}" >&2
    exit 2
  fi
}

validate_password() {
  local value="$1"
  if [[ -z "${value}" ]]; then
    echo "Password is required for --execute mode. Set --password or AOXC_TESTNET_PASSWORD." >&2
    exit 6
  fi
  if [[ "${#value}" -lt 16 ]]; then
    echo "Password must be at least 16 characters for production-grade testnet orchestration." >&2
    exit 6
  fi
  if [[ "${value}" =~ [[:space:]] ]]; then
    echo "Password must not contain whitespace." >&2
    exit 6
  fi
  if [[ ! "${value}" =~ [[:upper:]] ]] || [[ ! "${value}" =~ [[:lower:]] ]] || \
     [[ ! "${value}" =~ [0-9] ]] || [[ ! "${value}" =~ [^[:alnum:]] ]]; then
    echo "Password must include uppercase, lowercase, numeric, and symbol characters." >&2
    exit 6
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) ROOT_DIR="$2"; shift 2 ;;
    --validators) VALIDATORS="$2"; shift 2 ;;
    --bootnodes) BOOTNODES="$2"; shift 2 ;;
    --rpc) RPC_NODES="$2"; shift 2 ;;
    --archive) ARCHIVE_NODES="$2"; shift 2 ;;
    --sentry-per-validator) SENTRY_PER_VALIDATOR="$2"; shift 2 ;;
    --password) PASSWORD="$2"; shift 2 ;;
    --bind-host) BIND_HOST="$2"; shift 2 ;;
    --aoxc-bin) AOXC_BIN="$2"; shift 2 ;;
    --execute) EXECUTE=1; shift ;;
    --force) FORCE=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

require_uint "${VALIDATORS}" "--validators"
require_uint "${BOOTNODES}" "--bootnodes"
require_uint "${RPC_NODES}" "--rpc"
require_uint "${ARCHIVE_NODES}" "--archive"
require_uint "${SENTRY_PER_VALIDATOR}" "--sentry-per-validator"

if [[ "${VALIDATORS}" -lt 3 ]]; then
  echo "Testnet requires at least 3 validators." >&2
  exit 3
fi
if [[ "${BOOTNODES}" -lt 2 ]]; then
  echo "Testnet requires at least 2 bootnodes." >&2
  exit 3
fi

if [[ "${FORCE}" -eq 1 && -e "${ROOT_DIR}" ]]; then
  rm -rf "${ROOT_DIR}"
fi

if [[ -e "${ROOT_DIR}" ]]; then
  echo "Root already exists: ${ROOT_DIR} (use --force to replace)." >&2
  exit 4
fi

mkdir -p "${ROOT_DIR}/nodes" "${ROOT_DIR}/plans" "${ROOT_DIR}/logs"
chmod 700 "${ROOT_DIR}" "${ROOT_DIR}/nodes" "${ROOT_DIR}/plans" "${ROOT_DIR}/logs"

node_file="${ROOT_DIR}/plans/nodes.csv"
command_file="${ROOT_DIR}/plans/commands.sh"
summary_file="${ROOT_DIR}/plans/summary.txt"

cat > "${node_file}" <<CSV
name,role,home
CSV

cat > "${command_file}" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT

create_node() {
  local name="$1"
  local role="$2"
  local home="${ROOT_DIR}/nodes/${name}"

  mkdir -p "${home}"
  printf '%s,%s,%s\n' "${name}" "${role}" "${home}" >> "${node_file}"

  cat >> "${command_file}" <<SCRIPT
mkdir -p "${home}"
AOXC_HOME="${home}" "${AOXC_BIN}" production-bootstrap --profile "${PROFILE}" --password "\${AOXC_TESTNET_PASSWORD:?AOXC_TESTNET_PASSWORD is required}" --name "${name}" --bind-host "${BIND_HOST}"
AOXC_HOME="${home}" "${AOXC_BIN}" node start --home "${home}"
SCRIPT
}

for i in $(seq 1 "${VALIDATORS}"); do
  create_node "validator-${i}" "validator"
  for s in $(seq 1 "${SENTRY_PER_VALIDATOR}"); do
    create_node "validator-${i}-sentry-${s}" "sentry"
  done
done

for i in $(seq 1 "${BOOTNODES}"); do
  create_node "bootnode-${i}" "bootnode"
done

for i in $(seq 1 "${RPC_NODES}"); do
  create_node "rpc-${i}" "rpc"
done

for i in $(seq 1 "${ARCHIVE_NODES}"); do
  create_node "archive-${i}" "archive"
done

chmod +x "${command_file}"
chmod 700 "${command_file}"

total_nodes=$(($(wc -l < "${node_file}") - 1))
cat > "${summary_file}" <<SUMMARY
AOXC production-grade testnet plan
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
root=${ROOT_DIR}
profile=${PROFILE}
validators=${VALIDATORS}
bootnodes=${BOOTNODES}
rpc_nodes=${RPC_NODES}
archive_nodes=${ARCHIVE_NODES}
sentry_per_validator=${SENTRY_PER_VALIDATOR}
total_nodes=${total_nodes}
strict_testnet_minimum=validators>=3,bootnodes>=2
execute_mode=${EXECUTE}
SUMMARY
chmod 600 "${node_file}" "${summary_file}"

if [[ "${EXECUTE}" -eq 1 ]]; then
  validate_password "${PASSWORD}"
  if [[ ! -x "${AOXC_BIN}" ]]; then
    echo "AOXC binary is not executable: ${AOXC_BIN}" >&2
    exit 5
  fi
  AOXC_TESTNET_PASSWORD="${PASSWORD}" "${command_file}" | tee "${ROOT_DIR}/logs/execute.log"
fi

echo "Plan generated: ${summary_file}"
echo "Node inventory: ${node_file}"
echo "Bootstrap/start script: ${command_file}"
if [[ "${EXECUTE}" -eq 0 ]]; then
  echo "Dry-run mode active. Re-run with --execute to apply bootstrap/start commands."
fi
