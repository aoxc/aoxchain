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

usage() {
  cat <<USAGE
Usage: $(basename "$0") [options]

Provision a four-node AOXC local system layout with canonical genesis/config,
bootstrap each node, optionally run deterministic rounds, and produce snapshots.

Options:
  --root <path>           Output root (default: ${FULL_ROOT})
  --network <kind>        Environment kind under configs/environments (default: ${NETWORK_KIND})
  --rounds <n>            node-run rounds per node (default: ${ROUNDS})
  --skip-run              Bootstrap only; skip node-run
  --with-docker-assets    Generate docker-compose and Dockerfile assets
  --no-harden             Disable filesystem hardening chmod/chattr steps
  --force                 Remove existing root before provisioning
  -h, --help              Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
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

SOURCE_ROOT="${REPO_ROOT}/configs/environments/${NETWORK_KIND}"
[[ -d "${SOURCE_ROOT}" ]] || { echo "Missing source environment: ${SOURCE_ROOT}" >&2; exit 3; }

PROFILE_KIND="validation"
case "${NETWORK_KIND}" in
  mainnet) PROFILE_KIND="mainnet" ;;
  testnet) PROFILE_KIND="testnet" ;;
  validation|devnet|localnet|sovereign) PROFILE_KIND="validation" ;;
esac

for required in manifest.v1.json genesis.v1.json genesis.v1.sha256 validators.json bootnodes.json certificate.json profile.toml release-policy.toml; do
  [[ -f "${SOURCE_ROOT}/${required}" ]] || { echo "Missing required source file: ${SOURCE_ROOT}/${required}" >&2; exit 3; }
done

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

NODE_NAMES=(node1 node2 node3 node4)
for i in "${!NODE_NAMES[@]}"; do
  name="${NODE_NAMES[$i]}"
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

  password_file="${user_dir}/operator.password"
  printf 'AOXC-%s-LOCAL-PASSWORD-CHANGE-ME\n' "${name^^}" > "${password_file}"
  chmod 600 "${password_file}" || true

  AOXC_HOME="${home_dir}" "${AOXC_BIN_CMD[@]}" config-init --profile "${PROFILE_KIND}" --json-logs > "${FULL_ROOT}/nodes/${name}/run/config-init.json"
  AOXC_HOME="${home_dir}" "${AOXC_BIN_CMD[@]}" key-bootstrap --profile "${PROFILE_KIND}" --name "${name}" --password "$(cat "${password_file}")" > "${FULL_ROOT}/nodes/${name}/run/key-bootstrap.json"
  AOXC_HOME="${home_dir}" "${AOXC_BIN_CMD[@]}" keys-verify --password "$(cat "${password_file}")" > "${FULL_ROOT}/nodes/${name}/run/keys-verify.json"

  "${AOXC_BIN_CMD[@]}" node-bootstrap --home "${home_dir}" > "${FULL_ROOT}/nodes/${name}/run/bootstrap.json"

  if [[ "${RUN_AFTER_BOOTSTRAP}" == "1" ]]; then
    tx_prefix="${name^^}-TX"
    "${AOXC_BIN_CMD[@]}" node-run --home "${home_dir}" --rounds "${ROUNDS}" --sleep-ms 200 --tx-prefix "${tx_prefix}" > "${FULL_ROOT}/nodes/${name}/run/run.json"
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
root=${FULL_ROOT}
node_count=4
run_after_bootstrap=${RUN_AFTER_BOOTSTRAP}
rounds=${ROUNDS}
REPORT

(
  cd "${FULL_ROOT}"
  tar -czf "${FULL_ROOT}/system/snapshots/full-root-snapshot.tar.gz" nodes system users
)

if [[ "${WITH_DOCKER_ASSETS}" == "1" ]]; then
  cat > "${FULL_ROOT}/docker/Dockerfile.aoxc-local" <<'DOCKERFILE'
FROM rust:1.91-bookworm
WORKDIR /workspace/aoxchain
COPY . .
RUN cargo build --release -p aoxcmd --bin aoxc
ENTRYPOINT ["/workspace/aoxchain/target/release/aoxc"]
DOCKERFILE

  cat > "${FULL_ROOT}/docker/docker-compose.4nodes.yml" <<COMPOSE
services:
  aoxc-node1:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["node-run", "--home", "/data/node1/home", "--rounds", "${ROUNDS}", "--sleep-ms", "200", "--tx-prefix", "NODE1-TX"]
    volumes:
      - ${FULL_ROOT}/nodes/node1:/data/node1
  aoxc-node2:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["node-run", "--home", "/data/node2/home", "--rounds", "${ROUNDS}", "--sleep-ms", "200", "--tx-prefix", "NODE2-TX"]
    volumes:
      - ${FULL_ROOT}/nodes/node2:/data/node2
  aoxc-node3:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["node-run", "--home", "/data/node3/home", "--rounds", "${ROUNDS}", "--sleep-ms", "200", "--tx-prefix", "NODE3-TX"]
    volumes:
      - ${FULL_ROOT}/nodes/node3:/data/node3
  aoxc-node4:
    build:
      context: ${REPO_ROOT}
      dockerfile: ${FULL_ROOT}/docker/Dockerfile.aoxc-local
    command: ["node-run", "--home", "/data/node4/home", "--rounds", "${ROUNDS}", "--sleep-ms", "200", "--tx-prefix", "NODE4-TX"]
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
  fi
fi

echo "AOXC full 4-node layout created: ${FULL_ROOT}"
echo "Summary report: ${FULL_ROOT}/system/audit/provision-report.txt"
if [[ "${WITH_DOCKER_ASSETS}" == "1" ]]; then
  echo "Docker assets: ${FULL_ROOT}/docker/docker-compose.4nodes.yml"
fi
