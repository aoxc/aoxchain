#!/usr/bin/env bash
set -euo pipefail

echo "==> AOXC autonomy Make + sqlite verification"
TMP_HOME="$(mktemp -d)"
export HOME="${TMP_HOME}"
export AOXC_DATA_ROOT="${HOME}/.AOXCData"

echo "Using isolated HOME=${HOME}"

echo "==> Step 1: dry-run build matrix and publish commands"
make -n build-release-mainnet
make -n build-release-testnet
make -n build-release-devnet
make -n build-release-matrix
make -n publish-release

echo "==> Step 2: sqlite control-plane script lifecycle"
python3 ./scripts/autonomy_sqlite_ctl.py init
python3 ./scripts/autonomy_sqlite_ctl.py set-env --env mainnet --desired-state running --observed-state stopped --note "preflight"
python3 ./scripts/autonomy_sqlite_ctl.py event --env mainnet --action start --status ok --detail "manual verification"
python3 ./scripts/autonomy_sqlite_ctl.py release --version v0.0.0-test --artifact /tmp/aoxc-test.tar.gz --evidence /tmp/evidence.json
python3 ./scripts/autonomy_sqlite_ctl.py status
python3 ./scripts/autonomy_sqlite_ctl.py history --limit 5

echo "==> Step 3: make dry-run sqlite targets"
make -n db-init-sqlite
make -n db-status-sqlite
make -n db-event-sqlite
make -n db-release-sqlite
make -n db-history-sqlite

echo "Verification completed successfully."
