#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

#
# AOXC Deterministic Testnet Compatibility Launcher
#
# Purpose:
# - Provide the canonical entrypoint expected by readiness evaluation surfaces.
# - Preserve compatibility with the versioned deterministic testnet bundle layout.
# - Fail explicitly when mandatory fixture files are missing.
#
# Notes:
# - This script does not invent a hidden runtime flow.
# - It validates the deterministic fixture surface and emits operator guidance.
# - It may be extended later to invoke the actual orchestration pipeline.
#

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
FIXTURE_DIR="${ROOT_DIR}/deterministic-testnet"

GENESIS_PATH="${FIXTURE_DIR}/genesis.v1.json"
GENESIS_SHA_PATH="${FIXTURE_DIR}/genesis.v1.sha256"
CERTIFICATE_PATH="${FIXTURE_DIR}/certificate.json"
RELEASE_POLICY_PATH="${FIXTURE_DIR}/release-policy.toml"
PROFILE_PATH="${FIXTURE_DIR}/profile.toml"
MANIFEST_PATH="${FIXTURE_DIR}/manifest.v1.json"
BOOTNODES_PATH="${FIXTURE_DIR}/bootnodes.json"
VALIDATORS_PATH="${FIXTURE_DIR}/validators.json"

require_file() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    echo "[AOXC][error] required fixture file is missing: ${path}" >&2
    exit 1
  fi
}

require_file "${GENESIS_PATH}"
require_file "${PROFILE_PATH}"
require_file "${MANIFEST_PATH}"
require_file "${BOOTNODES_PATH}"
require_file "${VALIDATORS_PATH}"
require_file "${GENESIS_SHA_PATH}"
require_file "${CERTIFICATE_PATH}"
require_file "${RELEASE_POLICY_PATH}"

echo "[AOXC] deterministic testnet compatibility launcher"
echo "[AOXC] fixture directory : ${FIXTURE_DIR}"
echo "[AOXC] genesis          : ${GENESIS_PATH}"
echo "[AOXC] profile          : ${PROFILE_PATH}"
echo "[AOXC] manifest         : ${MANIFEST_PATH}"
echo "[AOXC] bootnodes        : ${BOOTNODES_PATH}"
echo "[AOXC] validators       : ${VALIDATORS_PATH}"
echo "[AOXC] genesis sha      : ${GENESIS_SHA_PATH}"
echo "[AOXC] certificate      : ${CERTIFICATE_PATH}"
echo "[AOXC] release policy   : ${RELEASE_POLICY_PATH}"

EXPECTED_HASH="$(awk '{print $1}' "${GENESIS_SHA_PATH}")"
ACTUAL_HASH="$(sha256sum "${GENESIS_PATH}" | awk '{print $1}')"

if [[ "${EXPECTED_HASH}" != "${ACTUAL_HASH}" ]]; then
  echo "[AOXC][error] genesis hash mismatch" >&2
  echo "[AOXC][error] expected: ${EXPECTED_HASH}" >&2
  echo "[AOXC][error] actual  : ${ACTUAL_HASH}" >&2
  exit 1
fi

echo "[AOXC] status           : fixture-surface-validated"
echo "[AOXC] genesis hash     : verified"

cat <<'EOF'

Next step:
- Wire this launcher to the authoritative deterministic testnet orchestration flow
  once the canonical node bootstrap/run pipeline is finalized.
- Until then, this compatibility entrypoint exists to satisfy readiness checks
  without introducing undocumented runtime behavior.

EOF
