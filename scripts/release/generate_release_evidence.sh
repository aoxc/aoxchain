#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="${ARTIFACT_DIR:-artifacts/release-evidence}"
RELEASE_ID="${RELEASE_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
AOXC_BIN="${AOXC_BIN:-cargo run -q -p aoxcmd --}"
SBOM_FILE="${ARTIFACT_DIR}/sbom-${RELEASE_ID}.json"
mkdir -p "${ARTIFACT_DIR}"

cargo fmt --all --check
cargo test -p aoxcmd -- --test-threads=1
cargo build --release -p aoxcmd --bin aoxc
sha256sum target/release/aoxc | tee "${ARTIFACT_DIR}/aoxc-${RELEASE_ID}.sha256"
${AOXC_BIN} build-manifest > "${ARTIFACT_DIR}/build-manifest-${RELEASE_ID}.json"
${AOXC_BIN} compat-matrix > "${ARTIFACT_DIR}/compat-matrix-${RELEASE_ID}.json"
${AOXC_BIN} production-audit --format json > "${ARTIFACT_DIR}/production-audit-${RELEASE_ID}.json"

if command -v cargo-cyclonedx >/dev/null 2>&1; then
  cargo cyclonedx --format json --output-file "${SBOM_FILE}"
else
  cargo metadata --format-version 1 --locked > /tmp/aoxc-release-metadata.json
  python3 - <<'PY' "${RELEASE_ID}" "${SBOM_FILE}"
import json
import sys

release_id, output_path = sys.argv[1], sys.argv[2]
with open("/tmp/aoxc-release-metadata.json", "r", encoding="utf-8") as handle:
    metadata = json.load(handle)

packages = []
for package in metadata.get("packages", []):
    packages.append(
        {
            "name": package["name"],
            "version": package["version"],
            "id": package["id"],
            "manifest_path": package["manifest_path"],
            "dependencies": sorted(dep["name"] for dep in package.get("dependencies", [])),
        }
    )

document = {
    "bomFormat": "AOXC-SBOM",
    "specVersion": "1.0",
    "serialNumber": f"urn:aoxc:sbom:{release_id}",
    "metadata": {
        "generated_by": "scripts/release/generate_release_evidence.sh",
        "source": "cargo metadata --format-version 1 --locked",
    },
    "components": packages,
}

with open(output_path, "w", encoding="utf-8") as handle:
    json.dump(document, handle, indent=2)
    handle.write("\n")
PY
fi

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
- artifact_sbom: sbom-${RELEASE_ID}.json
- artifact_signature: aoxc-${RELEASE_ID}.sig or .sig.status
- provenance_attestation: provenance-${RELEASE_ID}.json
- compatibility_matrix: compat-matrix-${RELEASE_ID}.json
- production_audit: production-audit-${RELEASE_ID}.json
- enforcement_rule: release is blocked if signature or provenance remains missing
REPORT

echo "[done] release evidence generated under ${ARTIFACT_DIR}"
