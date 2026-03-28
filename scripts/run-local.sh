#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

resolve_bin_path() {
  if [[ -n "${BIN_PATH:-}" && -x "${BIN_PATH}" ]]; then
    printf "%s" "${BIN_PATH}"
    return 0
  fi

  if [[ -x "${HOME}/.AOXCData/bin/aoxc" ]]; then
    printf "%s" "${HOME}/.AOXCData/bin/aoxc"
    return 0
  fi

  if [[ -x "./bin/aoxc" ]]; then
    printf "%s" "./bin/aoxc"
    return 0
  fi

  return 1
}

BIN_PATH="$(resolve_bin_path || true)"
if [[ -z "${BIN_PATH}" ]]; then
  echo "Missing AOXC binary. Run: make package-bin"
  echo "Expected paths: \$HOME/.AOXCData/bin/aoxc or ./bin/aoxc"
  exit 1
fi

"${BIN_PATH}" node-bootstrap
"${BIN_PATH}" produce-once --tx "local-smoke"
