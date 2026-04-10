#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# Produce cryptographically verifiable secure binary bundles.

set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

log() { printf '[secure-bundle] %s\n' "$*"; }
fail() { printf '[secure-bundle][error] %s\n' "$*" >&2; exit 2; }
require_cmd() { command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"; }

require_cmd openssl
require_cmd git
require_cmd python3

sha256_file() {
  local p="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${p}" | awk '{print $1}'
  else
    shasum -a 256 "${p}" | awk '{print $1}'
  fi
}

sha3_512_file() {
  local p="$1"
  openssl dgst -sha3-512 "${p}" | awk '{print $2}'
}

blake3_file() {
  local p="$1"
  if command -v b3sum >/dev/null 2>&1; then
    b3sum "${p}" | awk '{print $1}'
  else
    printf 'unavailable'
  fi
}

read_workspace_version() {
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' Cargo.toml | head -n1
}

RELEASE_VERSION="${RELEASE_VERSION:-$(read_workspace_version)}"
[[ -n "${RELEASE_VERSION}" ]] || fail "Unable to resolve RELEASE_VERSION"

OS_NAME="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH_NAME="$(uname -m | tr '[:upper:]' '[:lower:]')"
case "${ARCH_NAME}" in
  x86_64|amd64) ARCH_NAME="amd64" ;;
  aarch64|arm64) ARCH_NAME="arm64" ;;
esac

RELEASE_PLATFORM="${RELEASE_PLATFORM:-${OS_NAME}-${ARCH_NAME}}"
RELEASE_BIN_DIR="${RELEASE_BIN_DIR:-target/release}"
RELEASE_DIR="${RELEASE_DIR:-releases/v${RELEASE_VERSION}/${RELEASE_PLATFORM}}"
RELEASE_BINARIES=(aoxc aoxchub aoxckit)

RELEASE_SIGNING_KEY="${RELEASE_SIGNING_KEY:-}"
RELEASE_SIGNING_CERT="${RELEASE_SIGNING_CERT:-}"

[[ -f "${RELEASE_SIGNING_KEY}" ]] || fail "RELEASE_SIGNING_KEY file is required"
[[ -f "${RELEASE_SIGNING_CERT}" ]] || fail "RELEASE_SIGNING_CERT file is required"

mkdir -p "${RELEASE_DIR}/bin" "${RELEASE_DIR}/signatures"

manifest_file="${RELEASE_DIR}/manifest.secure.json"
sha256_file_out="${RELEASE_DIR}/SHA256SUMS"
sha3_file_out="${RELEASE_DIR}/SHA3-512SUMS"
blake3_file_out="${RELEASE_DIR}/BLAKE3SUMS"

: > "${sha256_file_out}"
: > "${sha3_file_out}"
: > "${blake3_file_out}"

cert_sha256="$(openssl x509 -in "${RELEASE_SIGNING_CERT}" -fingerprint -sha256 -noout | awk -F= '{print $2}' | tr -d ':')"

python3 - <<'PY' "${manifest_file}" "${RELEASE_VERSION}" "${RELEASE_PLATFORM}" "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)" "$(git rev-parse HEAD)" "${cert_sha256}"
import json, sys
manifest_path, version, platform, generated_at, git_commit, cert_fp = sys.argv[1:7]
manifest = {
    "release_version": version,
    "platform": platform,
    "generated_at_utc": generated_at,
    "git_commit": git_commit,
    "certificate": {"sha256_fingerprint": cert_fp},
    "security_profile": {
        "hash_algorithms": ["sha256", "sha3-512", "blake3(optional)"],
        "signature_algorithm": "RSA-SHA256",
        "posture": "defense-in-depth",
    },
    "artifacts": [],
}
with open(manifest_path, "w", encoding="utf-8") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")
PY

for bin in "${RELEASE_BINARIES[@]}"; do
  src="${RELEASE_BIN_DIR}/${bin}"
  [[ -f "${src}" ]] || fail "Missing binary: ${src}"
  dst="${RELEASE_DIR}/bin/${bin}"
  cp "${src}" "${dst}"
  chmod +x "${dst}" || true

  hash256="$(sha256_file "${dst}")"
  hash3="$(sha3_512_file "${dst}")"
  hashb3="$(blake3_file "${dst}")"

  printf '%s  %s\n' "${hash256}" "bin/${bin}" >> "${sha256_file_out}"
  printf '%s  %s\n' "${hash3}" "bin/${bin}" >> "${sha3_file_out}"
  printf '%s  %s\n' "${hashb3}" "bin/${bin}" >> "${blake3_file_out}"

  python3 - <<'PY' "${manifest_file}" "${bin}" "${hash256}" "${hash3}" "${hashb3}"
import json, sys
manifest_path, name, h256, h3, hb3 = sys.argv[1:6]
with open(manifest_path, "r", encoding="utf-8") as f:
    manifest = json.load(f)
manifest["artifacts"].append({
    "name": name,
    "path": f"bin/{name}",
    "sha256": h256,
    "sha3_512": h3,
    "blake3": hb3,
})
with open(manifest_path, "w", encoding="utf-8") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")
PY

done

openssl dgst -sha256 -sign "${RELEASE_SIGNING_KEY}" -out "${RELEASE_DIR}/signatures/manifest.secure.json.sig" "${manifest_file}"
openssl dgst -sha256 -sign "${RELEASE_SIGNING_KEY}" -out "${RELEASE_DIR}/signatures/SHA256SUMS.sig" "${sha256_file_out}"
openssl dgst -sha256 -sign "${RELEASE_SIGNING_KEY}" -out "${RELEASE_DIR}/signatures/SHA3-512SUMS.sig" "${sha3_file_out}"
openssl dgst -sha256 -sign "${RELEASE_SIGNING_KEY}" -out "${RELEASE_DIR}/signatures/BLAKE3SUMS.sig" "${blake3_file_out}"

log "Secure binary bundle created: ${RELEASE_DIR}"
log "Generated files: manifest.secure.json, SHA256SUMS, SHA3-512SUMS, BLAKE3SUMS, signatures/*"
