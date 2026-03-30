#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Purpose:
#   Generate a deterministic desktop operations report for operator review,
#   runtime visibility, and release-support documentation.
#
# Scope:
#   - Materialize a timestamped Markdown report beneath the repository artifact
#     surface
#   - Record environment lane command references
#   - Record wallet and critical CLI command references
#   - Capture a lightweight file presence snapshot for selected repository files
#
# Exit Codes:
#   0  Successful completion
#   2  Missing required host dependency
#   3  Output path preparation failure
#   4  Report generation failure
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly OUTPUT_DIR="${ROOT_DIR}/artifacts/desktop-ops"

TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
SAFE_TIMESTAMP="${TIMESTAMP//:/-}"
REPORT_FILE="${OUTPUT_DIR}/desktop-ops-report-${SAFE_TIMESTAMP}.md"

log_info() {
  printf '[info] %s\n' "$*"
}

log_error() {
  printf '[error] %s\n' "$*" >&2
}

die() {
  local message="$1"
  local exit_code="$2"
  log_error "${message}"
  exit "${exit_code}"
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}" 2
}

ensure_output_directory() {
  if [[ -e "${OUTPUT_DIR}" && ! -d "${OUTPUT_DIR}" ]]; then
    die "Output path exists but is not a directory: ${OUTPUT_DIR}" 3
  fi

  mkdir -p "${OUTPUT_DIR}" || die "Unable to create output directory: ${OUTPUT_DIR}" 3
}

append_file_presence_snapshot() {
  local check_file=''

  for check_file in \
    "AOXC_PROGRESS_REPORT.md" \
    "configs/mainnet.toml" \
    "configs/testnet.toml"; do
    if [[ -f "${ROOT_DIR}/${check_file}" ]]; then
      printf -- '- [x] %s\n' "${check_file}" >> "${REPORT_FILE}"
    else
      printf -- '- [ ] %s\n' "${check_file}" >> "${REPORT_FILE}"
    fi
  done
}

write_report() {
  cat > "${REPORT_FILE}" <<EOF
# AOXHub Desktop Ops Report

- Generated (UTC): ${TIMESTAMP}
- Repository Root: ${ROOT_DIR}

## Environment Lanes

| Lane | Chain ID | Command |
|---|---|---|
| Devnet | aoxc-devnet-local | \`cargo run -q -p aoxcmd -- devnet-up --profile local-dev\` |
| Testnet | aoxc-mainnet-candidate | \`configs/deterministic-testnet/launch-testnet.sh\` |
| Mainnet | aoxc-mainnet-candidate | \`cargo run -q -p aoxcmd -- production-audit --format json\` |

## Wallet Address Formation Commands

- \`cargo run -q -p aoxcmd -- wallet new-address --profile mainnet --lane operator\`
- \`cargo run -q -p aoxcmd -- wallet new-address --profile mainnet --lane treasury\`
- \`cargo run -q -p aoxcmd -- wallet inspect --profile mainnet --verbose\`

## Critical CLI Catalog

- \`cargo run -q -p aoxcmd -- node-run --home configs/mainnet-validator --rounds 12\`
- \`cargo run -q -p aoxcmd -- compat-matrix --format json\`
- \`cargo run -q -p aoxcmd -- production-audit --format json\`
- \`scripts/validation/network_production_closure.sh\`
- \`scripts/release/generate_release_evidence.sh\`

## File Presence Snapshot

EOF
}

main() {
  require_command date
  require_command mkdir
  require_command cat
  require_command printf

  ensure_output_directory

  log_info "Generating desktop operations report."
  write_report || die "Failed to write desktop operations report body." 4
  append_file_presence_snapshot || die "Failed to append file presence snapshot." 4

  printf 'Desktop ops report generated: %s\n' "${REPORT_FILE}"
}

main "$@"
