#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

: "${AOXC_BOOTSTRAP_PROFILE:=localnet}"
: "${AOXC_VALIDATOR_NAME:=validator-01}"
: "${AOXC_VALIDATOR_PASSWORD:=DemoPass#2026}"
: "${AOXC_NEW_ACCOUNT_ID:=AOXC_USER_ALICE}"
: "${AOXC_NEW_ACCOUNT_BALANCE:=1000000}"
: "${AOXC_TRANSFER_TO:=bob}"
: "${AOXC_TRANSFER_AMOUNT:=100}"

./scripts/preflight_check.sh
./scripts/chain_create.sh
./scripts/wallet_seed.sh
./scripts/network_start.sh
./scripts/finality_smoke.sh
./scripts/transfer_smoke.sh

cat <<OUT
AOXChain demo flow completed.
profile=${AOXC_BOOTSTRAP_PROFILE}
seeded_account=${AOXC_NEW_ACCOUNT_ID}
transfer_to=${AOXC_TRANSFER_TO}
transfer_amount=${AOXC_TRANSFER_AMOUNT}
OUT
