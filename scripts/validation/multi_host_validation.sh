#!/usr/bin/env bash
set -euo pipefail

# AOXC distributed validation harness.
# This script prepares a 3-5 host validation workflow but does not assume
# passwordless root or netem/iptables availability. Operators can plug in
# actual hostnames and fault hooks.

HOSTS_FILE="${HOSTS_FILE:-configs/deterministic-testnet/hosts.txt}"
HOSTS_TEMPLATE="${HOSTS_TEMPLATE:-configs/deterministic-testnet/hosts.txt.example}"
REMOTE_BASE="${REMOTE_BASE:-~/aoxc-distributed}"
FIXTURE_DIR="${FIXTURE_DIR:-configs/deterministic-testnet}"
ARTIFACT_DIR="${ARTIFACT_DIR:-artifacts/distributed-validation}"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
ROUNDS="${ROUNDS:-5}"
SLEEP_MS="${SLEEP_MS:-250}"

mkdir -p "${ARTIFACT_DIR}"

if [[ ! -f "${HOSTS_FILE}" ]]; then
  echo "[error] missing host file: ${HOSTS_FILE}" >&2
  if [[ -f "${HOSTS_TEMPLATE}" ]]; then
    echo "[hint] copy ${HOSTS_TEMPLATE} to ${HOSTS_FILE} and replace placeholders with 3-5 hosts" >&2
  else
    echo "[hint] create one hostname/IP per line for 3-5 hosts" >&2
  fi
  exit 2
fi

mapfile -t HOSTS < <(grep -v '^#' "${HOSTS_FILE}" | sed '/^$/d')
if [[ "${#HOSTS[@]}" -lt 3 ]]; then
  echo "[error] provide at least 3 hosts in ${HOSTS_FILE}" >&2
  exit 3
fi

for host in "${HOSTS[@]}"; do
  echo "[stage] sync fixture to ${host}"
  rsync -az "${FIXTURE_DIR}/" "${host}:${REMOTE_BASE}/fixture/"
done

idx=0
for host in "${HOSTS[@]}"; do
  node_name=$(printf '%s\n' atlas boreal cypher delta ember | sed -n "$((idx+1))p")
  if [[ -z "${node_name}" ]]; then
    break
  fi

  echo "[stage] bootstrap ${node_name} on ${host}"
  ssh "${host}" "cd ${REMOTE_BASE} && ${AOXC_BIN} node-bootstrap --home fixture/homes/${node_name}" \
    | tee "${ARTIFACT_DIR}/${node_name}-bootstrap.json"

  echo "[stage] short run ${node_name} on ${host}"
  ssh "${host}" "cd ${REMOTE_BASE} && ${AOXC_BIN} node-run --home fixture/homes/${node_name} --rounds ${ROUNDS} --sleep-ms ${SLEEP_MS} --tx-prefix ${node_name^^}-DIST" \
    | tee "${ARTIFACT_DIR}/${node_name}-run.json"

  idx=$((idx+1))
done

cat > "${ARTIFACT_DIR}/REPORT_TEMPLATE.md" <<'REPORT'
# Distributed Validation Report

## Amaç

## Kurulum

## Test adımları

## Gözlem

## Sorunlar

## Sonuç

## Sonraki adım

## Genel not
REPORT

echo "[done] artifacts written under ${ARTIFACT_DIR}"
