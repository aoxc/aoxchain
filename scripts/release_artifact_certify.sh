#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

# Generate deterministic release artifact hash manifests and optional detached
# signatures for operator / auditor verification.
#
# Usage:
#   ./scripts/release_artifact_certify.sh target/release/aoxc target/release/aoxckit
#   AOXC_RELEASE_SIGNING_KEY_PEM=key.pem \
#   AOXC_RELEASE_SIGNING_CERT_PEM=cert.pem \
#   ./scripts/release_artifact_certify.sh target/release/aoxc

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <artifact> [<artifact> ...]" >&2
  exit 2
fi

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[error] missing required command: $cmd" >&2
    exit 127
  fi
}

require_cmd sha256sum
require_cmd python3

OUT_DIR="${AOXC_RELEASE_ARTIFACT_DIR:-./dist/release-artifacts}"
MANIFEST_PATH="${OUT_DIR}/artifact-manifest.json"
CHECKSUMS_PATH="${OUT_DIR}/SHA256SUMS"
SIGNATURE_PATH="${OUT_DIR}/artifact-manifest.sig"
CERT_COPY_PATH="${OUT_DIR}/artifact-signing-cert.pem"

mkdir -p "${OUT_DIR}"
: > "${CHECKSUMS_PATH}"

artifacts_json="[]"

for artifact in "$@"; do
  if [[ ! -f "${artifact}" ]]; then
    echo "[error] artifact does not exist: ${artifact}" >&2
    exit 3
  fi

  digest="$(sha256sum "${artifact}" | awk '{print $1}')"
  size="$(stat -c %s "${artifact}")"
  filename="$(basename "${artifact}")"

  printf '%s  %s\n' "${digest}" "${filename}" >> "${CHECKSUMS_PATH}"

  artifacts_json="$(
    python3 - <<'PY' "${artifacts_json}" "${artifact}" "${filename}" "${digest}" "${size}"
import json
import sys

current = json.loads(sys.argv[1])
current.append(
    {
        "path": sys.argv[2],
        "file_name": sys.argv[3],
        "sha256": sys.argv[4],
        "size_bytes": int(sys.argv[5]),
    }
)
print(json.dumps(current, separators=(",", ":")))
PY
  )"
done

python3 - <<'PY' "${artifacts_json}" "${MANIFEST_PATH}"
import json
import os
import sys
from datetime import datetime, timezone

manifest = {
    "schema_version": 1,
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "artifacts": json.loads(sys.argv[1]),
}
with open(sys.argv[2], "w", encoding="utf-8") as fh:
    json.dump(manifest, fh, indent=2, sort_keys=True)
    fh.write("\n")
PY

if [[ -n "${AOXC_RELEASE_SIGNING_KEY_PEM:-}" || -n "${AOXC_RELEASE_SIGNING_CERT_PEM:-}" ]]; then
  require_cmd openssl

  if [[ -z "${AOXC_RELEASE_SIGNING_KEY_PEM:-}" || -z "${AOXC_RELEASE_SIGNING_CERT_PEM:-}" ]]; then
    echo "[error] both AOXC_RELEASE_SIGNING_KEY_PEM and AOXC_RELEASE_SIGNING_CERT_PEM are required for signing" >&2
    exit 4
  fi

  if [[ ! -f "${AOXC_RELEASE_SIGNING_KEY_PEM}" || ! -f "${AOXC_RELEASE_SIGNING_CERT_PEM}" ]]; then
    echo "[error] signing key or certificate path does not exist" >&2
    exit 5
  fi

  openssl dgst -sha256 -sign "${AOXC_RELEASE_SIGNING_KEY_PEM}" \
    -out "${SIGNATURE_PATH}" "${MANIFEST_PATH}"
  cp "${AOXC_RELEASE_SIGNING_CERT_PEM}" "${CERT_COPY_PATH}"
fi

echo "[ok] wrote ${MANIFEST_PATH}"
echo "[ok] wrote ${CHECKSUMS_PATH}"
if [[ -f "${SIGNATURE_PATH}" ]]; then
  echo "[ok] wrote ${SIGNATURE_PATH}"
  echo "[ok] copied ${CERT_COPY_PATH}"
fi
