#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/runtime_common.sh"

run_make_target ops-stop || true
run_make_target runtime-reinstall
run_make_target ops-start
