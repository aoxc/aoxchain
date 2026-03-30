#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/artifacts/desktop-ops"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
REPORT_FILE="${OUTPUT_DIR}/desktop-ops-report-${TIMESTAMP//[:]/-}.md"

mkdir -p "${OUTPUT_DIR}"

cat > "${REPORT_FILE}" <<REPORT
# AOXHub Desktop Ops Report

- Generated (UTC): ${TIMESTAMP}
- Repository: ${ROOT_DIR}

## Environment Lanes

| Lane | Chain ID | Command |
|---|---|---|
| Devnet | aoxc-devnet-local | \
\`cargo run -q -p aoxcmd -- devnet-up --profile local-dev\` |
| Testnet | aoxc-mainnet-candidate | \
\`configs/deterministic-testnet/launch-testnet.sh\` |
| Mainnet | aoxc-mainnet-candidate | \
\`cargo run -q -p aoxcmd -- production-audit --format json\` |

## Wallet Address Formation Commands

- \
\`cargo run -q -p aoxcmd -- wallet new-address --profile mainnet --lane operator\`
- \
\`cargo run -q -p aoxcmd -- wallet new-address --profile mainnet --lane treasury\`
- \
\`cargo run -q -p aoxcmd -- wallet inspect --profile mainnet --verbose\`

## Critical CLI Catalog

- \
\`cargo run -q -p aoxcmd -- node-run --home configs/mainnet-validator --rounds 12\`
- \
\`cargo run -q -p aoxcmd -- compat-matrix --format json\`
- \
\`cargo run -q -p aoxcmd -- production-audit --format json\`
- \
\`scripts/validation/network_production_closure.sh\`
- \
\`scripts/release/generate_release_evidence.sh\`

## File Presence Snapshot

REPORT

for check_file in \
  "AOXC_PROGRESS_REPORT.md" \
  "configs/mainnet.toml" \
  "configs/testnet.toml"; do
  if [[ -f "${ROOT_DIR}/${check_file}" ]]; then
    echo "- [x] ${check_file}" >> "${REPORT_FILE}"
  else
    echo "- [ ] ${check_file}" >> "${REPORT_FILE}"
  fi
done

echo
printf 'Desktop ops report generated: %s\n' "${REPORT_FILE}"
