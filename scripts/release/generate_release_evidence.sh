#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-artifacts/release-evidence}"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
mkdir -p "${ARTIFACT_DIR}"

cargo fmt --all --check
cargo test -p aoxcmd -- --test-threads=1
cargo build --release -p aoxcmd --bin aoxc
sha256sum target/release/aoxc | tee "${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sha256"
${AOXC_BIN} build-manifest > "${ARTIFACT_DIR}/build-manifest-${RELEASE_ID}.json"
${AOXC_BIN} compat-matrix > "${ARTIFACT_DIR}/compat-matrix-${RELEASE_ID}.json"
${AOXC_BIN} production-audit --format json > "${ARTIFACT_DIR}/production-audit-${RELEASE_ID}.json"

if [[ -n "${AOXC_SIGNING_CMD:-}" ]]; then
  eval "${AOXC_SIGNING_CMD} target/release/aoxc > ${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sig"
else
  printf 'MISSING_SIGNATURE\n' > "${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sig.status"
fi

if [[ -n "${AOXC_PROVENANCE_CMD:-}" ]]; then
  eval "${AOXC_PROVENANCE_CMD} target/release/aoxc > ${ARTIFACT_DIR}/provenance-${RELEASE_ID}.json"
else
  cat > "${ARTIFACT_DIR}/provenance-${RELEASE_ID}.json" <<JSON
{
  "status": "missing-generator",
  "requirement": "Set AOXC_PROVENANCE_CMD to emit a provenance attestation before release approval"
}
JSON
fi

cat > "${ARTIFACT_DIR}/release-evidence-${RELEASE_ID}.md" <<REPORT
# AOXC Release Evidence ${RELEASE_ID}

- reproducible_build: cargo fmt --all --check && cargo test -p aoxcmd && cargo build --release -p aoxcmd --bin aoxc
- artifact_checksum: aoxc-${RELEASE_ID}.sha256
- artifact_signature: aoxc-${RELEASE_ID}.sig or .sig.status
- provenance_attestation: provenance-${RELEASE_ID}.json
- compatibility_matrix: compat-matrix-${RELEASE_ID}.json
- production_audit: production-audit-${RELEASE_ID}.json
- enforcement_rule: release is blocked if signature or provenance remains missing
REPORT

echo "[done] release evidence generated under ${ARTIFACT_DIR}"
