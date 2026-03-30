#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

NETWORK_KIND="${AOXC_FULL_NETWORK_KIND:-localnet}"
FULL_ROOT="${AOXC_FULL_ROOT:-${HOME}/.aoxc-full-4nodes}"
ROUNDS="${AOXC_FULL_ROUNDS:-3}"
RUN_AFTER_BOOTSTRAP="${AOXC_FULL_RUN_AFTER_BOOTSTRAP:-1}"
WITH_DOCKER_ASSETS="${AOXC_FULL_WITH_DOCKER_ASSETS:-0}"
HARDEN_LAYOUT="${AOXC_FULL_HARDEN_LAYOUT:-1}"
FORCE_RESET="${AOXC_FULL_FORCE_RESET:-0}"
MODE="apply"
PLAN_OUT=""
PASSWORD_SOURCE="${AOXC_FULL_PASSWORD_SOURCE:-generated}" # generated|env|file|inline
PASSWORD_FILE="${AOXC_FULL_PASSWORD_FILE:-}"
PASSWORD_INLINE="${AOXC_FULL_PASSWORD_INLINE:-}"
PASSWORD_ENV_PREFIX="${AOXC_FULL_PASSWORD_ENV_PREFIX:-AOXC_NODE_PASSWORD_}"

NODE_NAMES=(node1 node2 node3 node4)

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Provision a four-node AOXC local system layout with canonical genesis/config,
bootstrap each node, optionally run deterministic rounds, and produce snapshots.

Modes:
  --plan                 Print planned actions only (no writes)
  --apply                Execute provisioning (default)

Options:
  --root <path>          Output root (default: ${FULL_ROOT})
  --network <kind>       Environment kind under configs/environments (default: ${NETWORK_KIND})
  --rounds <n>           node-run rounds per node (default: ${ROUNDS})
  --skip-run             Bootstrap only; skip node-run
  --with-docker-assets   Generate docker-compose and Dockerfile assets
  --no-harden            Disable filesystem hardening chmod/chattr steps
  --force                Remove existing root before provisioning
  --plan-out <path>      Save plan output to file (with --plan)

Password sources (per node):
  --password-source generated|env|file|inline
  --password-file <path>    File format: node1=password
  --password-inline <spec>  Format: node1=...;node2=...;node3=...;node4=...

  -h, --help            Show this help
USAGE
}

log_step() {
  printf '[aoxc-full-4nodes] %s\n' "$*"
}

plan_step() {
  local line="$*"
  echo "${line}"
  if [[ -n "${PLAN_OUT}" ]]; then
    echo "${line}" >> "${PLAN_OUT}"
  fi
}

ensure_tools() {
  command -v sha256sum >/dev/null 2>&1 || {
    echo "Missing required command: sha256sum" >&2
    exit 2
  }
  command -v tar >/dev/null 2>&1 || {
    echo "Missing required command: tar" >&2
    exit 2
  }
}

profile_for_network() {
  case "$1" in
    mainnet) echo "mainnet" ;;
    testnet) echo "testnet" ;;
    validation|devnet|localnet|sovereign) echo "validation" ;;
    *) echo "validation" ;;
  esac
}

load_password_file() {
  local path="$1"
  [[ -f "${path}" ]] || {
    echo "Password file not found: ${path}" >&2
    exit 2
  }
  PASSWORD_INLINE="$(awk -F= 'NF>=2 {gsub(/^[ \t]+|[ \t]+$/, "", $1); sub(/^[ \t]+/, "", $2); printf "%s=%s;", $1, $2}' "${path}")"
}

resolve_password_from_inline() {
  local node="$1"
  local token
  IFS=';' read -r -a token <<< "${PASSWORD_INLINE}"
  for entry in "${token[@]}"; do
    [[ -z "${entry}" ]] && continue
    local key="${entry%%=*}"
    local val="${entry#*=}"
    if [[ "${key}" == "${node}" ]]; then
      printf '%s' "${val}"
      return 0
    fi
  done
  return 1
}

resolve_node_password() {
  local node="$1"
  case "${PASSWORD_SOURCE}" in
    generated)
      printf 'AOXC-%s-LOCAL-PASSWORD-CHANGE-ME' "${node^^}"
      ;;
    env)
      local env_name="${PASSWORD_ENV_PREFIX}${node^^}"
      local value="${!env_name:-}"
      [[ -n "${value}" ]] || {
        echo "Missing environment password: ${env_name}" >&2
        exit 2
      }
      printf '%s' "${value}"
      ;;
    file)
      [[ -n "${PASSWORD_FILE}" ]] || {
        echo "--password-file is required when --password-source file" >&2
        exit 2
      }
      if [[ -z "${PASSWORD_INLINE}" ]]; then
        load_password_file "${PASSWORD_FILE}"
      fi
      resolve_password_from_inline "${node}" || {
        echo "Password missing for ${node} in ${PASSWORD_FILE}" >&2
        exit 2
      }
      ;;
    inline)
      [[ -n "${PASSWORD_INLINE}" ]] || {
        echo "--password-inline is required when --password-source inline" >&2
        exit 2
      }
      resolve_password_from_inline "${node}" || {
        echo "Password missing for ${node} in --password-inline" >&2
        exit 2
      }
      ;;
    *)
      echo "Unsupported password source: ${PASSWORD_SOURCE}" >&2
      exit 2
      ;;
  esac
}

create_snapshot_manifest() {
  local root="$1"
  local manifest_file="${root}/system/snapshots/SHA256SUMS"
  : > "${manifest_file}"
  (
    cd "${root}"
    while IFS= read -r path; do
      sha256sum "${path}" >> "${manifest_file}"
    done < <(find nodes system/snapshots -type f \( -name '*.tar.gz' -o -name '*.json' -o -name '*.prom' \) | sort)
  )
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --plan)
      MODE="plan"
      shift
      ;;
    --apply)
      MODE="apply"
      shift
      ;;
    --plan-out)
      PLAN_OUT="$2"
      shift 2
      ;;
    --root)
      FULL_ROOT="$2"
      shift 2
      ;;
    --network)
      NETWORK_KIND="$2"
      shift 2
      ;;
    --rounds)
      ROUNDS="$2"
      shift 2
      ;;
    --skip-run)
      RUN_AFTER_BOOTSTRAP=0
      shift
      ;;
    --with-docker-assets)
      WITH_DOCKER_ASSETS=1
      shift
      ;;
    --no-harden)
      HARDEN_LAYOUT=0
      shift
      ;;
    --force)
      FORCE_RESET=1
      shift
      ;;
    --password-source)
      PASSWORD_SOURCE="$2"
      shift 2
      ;;
    --password-file)
      PASSWORD_FILE="$2"
      shift 2
      ;;
    --password-inline)
      PASSWORD_INLINE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if ! [[ "${ROUNDS}" =~ ^[0-9]+$ ]] || [[ "${ROUNDS}" -eq 0 ]]; then
  echo "--rounds must be a positive integer" >&2
  exit 2
fi

ensure_tools
SOURCE_ROOT="${REPO_ROOT}/configs/environments/${NETWORK_KIND}"
[[ -d "${SOURCE_ROOT}" ]] || { echo "Missing source environment: ${SOURCE_ROOT}" >&2; exit 3; }
PROFILE_KIND="$(profile_for_network "${NETWORK_KIND}")"

for required in manifest.v1.json genesis.v1.json genesis.v1.sha256 validators.json bootnodes.json certificate.json profile.toml release-policy.toml; do
  [[ -f "${SOURCE_ROOT}/${required}" ]] || { echo "Missing required source file: ${SOURCE_ROOT}/${required}" >&2; exit 3; }
done

if [[ "${MODE}" == "plan" ]]; then
  [[ -z "${PLAN_OUT}" ]] || : > "${PLAN_OUT}"
  plan_step "MODE=plan"
  plan_step "FULL_ROOT=${FULL_ROOT}"
  plan_step "NETWORK_KIND=${NETWORK_KIND}"
  plan_step "PROFILE_KIND=${PROFILE_KIND}"
  plan_step "RUN_AFTER_BOOTSTRAP=${RUN_AFTER_BOOTSTRAP}"
  plan_step "WITH_DOCKER_ASSETS=${WITH_DOCKER_ASSETS}"
  plan_step "HARDEN_LAYOUT=${HARDEN_LAYOUT}"
  plan_step "PASSWORD_SOURCE=${PASSWORD_SOURCE}"
  plan_step "Create directories: system/* users/* nodes/node{1..4}"
  plan_step "Copy canonical materials from ${SOURCE_ROOT}"
  plan_step "Per node: config-init, key-bootstrap, keys-verify, node-bootstrap, optional node-run"
  plan_step "Produce per-node snapshots and full root snapshot"
  plan_step "Produce snapshot SHA256 manifest"
  [[ "${WITH_DOCKER_ASSETS}" == "1" ]] && plan_step "Generate Dockerfile and docker-compose.4nodes.yml"
  [[ "${HARDEN_LAYOUT}" == "1" ]] && plan_step "Apply chmod and optional chattr immutable flags"
  exit 0
fi

if [[ "${FORCE_RESET}" == "1" && -e "${FULL_ROOT}" ]]; then
  chmod -R u+w "${FULL_ROOT}" 2>/dev/null || true
  chattr -R -i "${FULL_ROOT}" 2>/dev/null || true
  rm -rf "${FULL_ROOT}"
fi

if [[ -e "${FULL_ROOT}" ]]; then
  echo "Target root already exists: ${FULL_ROOT}" >&2
  echo "Use --force to replace it." >&2
  exit 4
fi

mkdir -p \
  "${FULL_ROOT}/system/genesis" \
  "${FULL_ROOT}/system/config" \
  "${FULL_ROOT}/system/snapshots" \
  "${FULL_ROOT}/system/logs" \
  "${FULL_ROOT}/system/audit" \
  "${FULL_ROOT}/users/operator" \
  "${FULL_ROOT}/docker" \
  "${FULL_ROOT}/nodes"

cp "${SOURCE_ROOT}/manifest.v1.json" "${FULL_ROOT}/system/genesis/manifest.json"
cp "${SOURCE_ROOT}/genesis.v1.json" "${FULL_ROOT}/system/genesis/genesis.json"
cp "${SOURCE_ROOT}/genesis.v1.sha256" "${FULL_ROOT}/system/genesis/genesis.sha256"
cp "${SOURCE_ROOT}/validators.json" "${FULL_ROOT}/system/config/validators.json"
cp "${SOURCE_ROOT}/bootnodes.json" "${FULL_ROOT}/system/config/bootnodes.json"
cp "${SOURCE_ROOT}/certificate.json" "${FULL_ROOT}/system/config/certificate.json"
cp "${SOURCE_ROOT}/profile.toml" "${FULL_ROOT}/system/config/profile.toml"
cp "${SOURCE_ROOT}/release-policy.toml" "${FULL_ROOT}/system/config/release-policy.toml"

AOXC_BIN_CMD=( )
if [[ -x "${REPO_ROOT}/target/release/aoxc" ]]; then
  AOXC_BIN_CMD=("${REPO_ROOT}/target/release/aoxc")
else
  AOXC_BIN_CMD=(cargo run -q -p aoxcmd --)
fi

for name in "${NODE_NAMES[@]}"; do
  log_step "Provisioning ${name}"
  home_dir="${FULL_ROOT}/nodes/${name}/home"
  user_dir="${FULL_ROOT}/users/${name}"

  mkdir -p \
    "${home_dir}/identity" \
    "${home_dir}/config" \
    "${home_dir}/runtime" \
    "${home_dir}/logs" \
    "${home_dir}/audit" \
    "${FULL_ROOT}/nodes/${name}/snapshots" \
    "${FULL_ROOT}/nodes/${name}/run" \
    "${FULL_ROOT}/nodes/${name}/evidence" \
    "${user_dir}"

  cp "${FULL_ROOT}/system/genesis/genesis.json" "${home_dir}/identity/genesis.json"
  cp "${FULL_ROOT}/system/config/profile.toml" "${home_dir}/config/profile.toml"
  cp "${FULL_ROOT}/system/config/validators.json" "${home_dir}/config/validators.json"
  cp "${FULL_ROOT}/system/config/bootnodes.json" "${home_dir}/config/bootnodes.json"

  node_password="$(resolve_node_password "${name}")"
  password_file="${user_dir}/operator.password"
  printf '%s\n' "${node_password}" > "${password_file}"
  chmod 600 "${password_file}" || true

  AOXC_HOME="${home_dir}" "${AOXC_BIN_CMD[@]}" config-init --profile "${PROFILE_KIND}" --json-logs > "${FULL_ROOT}/nodes/${name}/run/config-init.json"
  AOXC_HOME="${home_dir}" "${AOXC_BIN_CMD[@]}" key-bootstrap --profile "${PROFILE_KIND}" --name "${name}" --password "${node_password}" > "${FULL_ROOT}/nodes/${name}/run/key-bootstrap.json"
  AOXC_HOME="${home_dir}" "${AOXC_BIN_CMD[@]}" keys-verify --password "${node_password}" > "${FULL_ROOT}/nodes/${name}/run/keys-verify.json"

  "${AOXC_BIN_CMD[@]}" node-bootstrap --home "${home_dir}" > "${FULL_ROOT}/nodes/${name}/run/bootstrap.json"

  if [[ "${RUN_AFTER_BOOTSTRAP}" == "1" ]]; then
    tx_prefix="${name^^}-TX"
    "${AOXC_BIN_CMD[@]}" node-run --home "${home_dir}" --rounds "${ROUNDS}" --sleep-ms 200 --tx-prefix "${tx_prefix}" --no-live-log > "${FULL_ROOT}/nodes/${name}/run/run.json"
  fi

  tar -C "${home_dir}" -czf "${FULL_ROOT}/nodes/${name}/snapshots/${name}-home.tar.gz" .
  cp "${home_dir}/runtime/state.json" "${FULL_ROOT}/nodes/${name}/snapshots/state.json" 2>/dev/null || true
  cp "${home_dir}/runtime/metrics.prom" "${FULL_ROOT}/nodes/${name}/snapshots/metrics.prom" 2>/dev/null || true
  cp "${home_dir}/runtime/metrics.json" "${FULL_ROOT}/nodes/${name}/snapshots/metrics.json" 2>/dev/null || true
done

cat > "${FULL_ROOT}/system/audit/provision-report.txt" <<REPORT
AOXC full 4-node provision report
created_utc=$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)
repo_root=${REPO_ROOT}
network_kind=${NETWORK_KIND}
profile_kind=${PROFILE_KIND}
root=${FULL_ROOT}
node_count=4
run_after_bootstrap=${RUN_AFTER_BOOTSTRAP}
rounds=${ROUNDS}
password_source=${PASSWORD_SOURCE}
REPORT

(
  cd "${FULL_ROOT}"
  tar -czf "${FULL_ROOT}/system/snapshots/full-root-snapshot.tar.gz" nodes system users
)

create_snapshot_manifest "${FULL_ROOT}"

if [[ "${WITH_DOCKER_ASSETS}" == "1" ]]; then
  cat > "${FULL_ROOT}/docker/Dockerfile.aoxc-local" <<'DOCKERFILE'
FROM rust:1.91-bookworm
WORKDIR /workspace/aoxchain
COPY . .
RUN cargo build --release -p aoxcmd --bin aoxc
CMD ["bash", "-lc", "echo 'Use docker-compose.4nodes.yml services to run nodes.'"]
DOCKERFILE

  cat > "${FULL_ROOT}/docker/docker-compose.4nodes.yml" <<COMPOSE
services:
  aoxc-node1:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["bash", "-lc", "target/release/aoxc node-bootstrap --home /data/node1/home && while true; do target/release/aoxc node-run --home /data/node1/home --rounds 1 --sleep-ms 200 --tx-prefix NODE1-TX --no-live-log; sleep 1; done"]
    healthcheck:
      test: ["CMD", "bash", "-lc", "test -f /data/node1/home/runtime/state.json"]
      interval: 10s
      timeout: 3s
      retries: 10
    volumes:
      - ${FULL_ROOT}/nodes/node1:/data/node1
  aoxc-node2:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["bash", "-lc", "target/release/aoxc node-bootstrap --home /data/node2/home && while true; do target/release/aoxc node-run --home /data/node2/home --rounds 1 --sleep-ms 200 --tx-prefix NODE2-TX --no-live-log; sleep 1; done"]
    healthcheck:
      test: ["CMD", "bash", "-lc", "test -f /data/node2/home/runtime/state.json"]
      interval: 10s
      timeout: 3s
      retries: 10
    volumes:
      - ${FULL_ROOT}/nodes/node2:/data/node2
  aoxc-node3:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["bash", "-lc", "target/release/aoxc node-bootstrap --home /data/node3/home && while true; do target/release/aoxc node-run --home /data/node3/home --rounds 1 --sleep-ms 200 --tx-prefix NODE3-TX --no-live-log; sleep 1; done"]
    healthcheck:
      test: ["CMD", "bash", "-lc", "test -f /data/node3/home/runtime/state.json"]
      interval: 10s
      timeout: 3s
      retries: 10
    volumes:
      - ${FULL_ROOT}/nodes/node3:/data/node3
  aoxc-node4:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["bash", "-lc", "target/release/aoxc node-bootstrap --home /data/node4/home && while true; do target/release/aoxc node-run --home /data/node4/home --rounds 1 --sleep-ms 200 --tx-prefix NODE4-TX --no-live-log; sleep 1; done"]
    healthcheck:
      test: ["CMD", "bash", "-lc", "test -f /data/node4/home/runtime/state.json"]
      interval: 10s
      timeout: 3s
      retries: 10
    volumes:
      - ${FULL_ROOT}/nodes/node4:/data/node4
COMPOSE
fi

if [[ "${HARDEN_LAYOUT}" == "1" ]]; then
  chmod -R go-rwx "${FULL_ROOT}" || true
  chmod -R u+rwX "${FULL_ROOT}" || true
  if command -v chattr >/dev/null 2>&1; then
    chattr +i "${FULL_ROOT}/system/audit/provision-report.txt" 2>/dev/null || true
    chattr +i "${FULL_ROOT}/system/genesis/genesis.json" 2>/dev/null || true
    chattr +i "${FULL_ROOT}/system/genesis/genesis.sha256" 2>/dev/null || true
    chattr +i "${FULL_ROOT}/system/snapshots/SHA256SUMS" 2>/dev/null || true
  fi
fi

log_step "AOXC full 4-node layout created: ${FULL_ROOT}"
log_step "Summary report: ${FULL_ROOT}/system/audit/provision-report.txt"
log_step "Snapshot manifest: ${FULL_ROOT}/system/snapshots/SHA256SUMS"
if [[ "${WITH_DOCKER_ASSETS}" == "1" ]]; then
  log_step "Docker assets: ${FULL_ROOT}/docker/docker-compose.4nodes.yml"
fi
