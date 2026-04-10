#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

log() {
  echo "[cargo-deny-gate] $*"
}

fail() {
  echo "[cargo-deny-gate][error] $*" >&2
  exit 1
}

if ! command -v cargo-deny >/dev/null 2>&1; then
  fail "cargo-deny is not installed. Install with: cargo install cargo-deny"
fi

log "Running cargo deny policy checks"
cargo deny check

log "cargo deny policy checks passed"
