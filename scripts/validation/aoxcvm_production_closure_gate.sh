#!/usr/bin/env bash
set -euo pipefail

# AOXCVM production closure evidence gate.
# Default mode reports gaps but does not fail.
# Set AOXCVM_PRODUCTION_CLOSURE_STRICT=1 to fail on any missing item.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STRICT="${AOXCVM_PRODUCTION_CLOSURE_STRICT:-0}"

required_files=(
  "crates/aoxcvm/docs/CRYPTO_HASH_POSTURE.md"
  "crates/aoxcvm/docs/FINGERPRINT_SPEC.md"
  "crates/aoxcvm/docs/PRODUCTION_CLOSURE_MASTER_PLAN.md"
  "crates/aoxcvm/docs/PHASE3_RELEASE_CLOSURE.md"
)

required_paths=(
  "artifacts/aoxcvm-phase3"
  "artifacts/release-evidence"
)

missing=0

echo "[aoxcvm-production-closure] scanning required files..."
for path in "${required_files[@]}"; do
  if [[ -f "${ROOT_DIR}/${path}" ]]; then
    echo "  [ok] ${path}"
  else
    echo "  [missing] ${path}"
    missing=1
  fi
done

echo "[aoxcvm-production-closure] scanning required paths..."
for path in "${required_paths[@]}"; do
  if [[ -e "${ROOT_DIR}/${path}" ]]; then
    echo "  [ok] ${path}"
  else
    echo "  [missing] ${path}"
    missing=1
  fi
done

if [[ "${missing}" -ne 0 ]]; then
  if [[ "${STRICT}" == "1" ]]; then
    echo "[aoxcvm-production-closure] FAILED (strict mode): evidence gaps detected."
    exit 1
  fi
  echo "[aoxcvm-production-closure] WARNING: evidence gaps detected (non-strict mode)."
  exit 0
fi

echo "[aoxcvm-production-closure] PASS: baseline evidence surfaces are present."
