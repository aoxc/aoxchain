#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

echo "[deprecated] scripts/aoxc-q-v0.2.0.start.sh has been replaced by scripts/aoxc-rolling.start.sh" >&2
exec "${SCRIPT_DIR}/aoxc-rolling.start.sh" "$@"
