#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# Verify secure binary bundles with signatures and multi-hash checks.

set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

log() { printf '[verify-secure-bundle] %s\n' "$*"; }
fail() { printf '[verify-secure-bundle][error] %s\n' "$*" >&2; exit 2; }
require_cmd() { command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"; }

require_cmd openssl
require_cmd python3

read_workspace_version() {
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' Cargo.toml | head -n1
}

RELEASE_VERSION="${RELEASE_VERSION:-$(read_workspace_version)}"
OS_NAME="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH_NAME="$(uname -m | tr '[:upper:]' '[:lower:]')"
case "${ARCH_NAME}" in
  x86_64|amd64) ARCH_NAME="amd64" ;;
  aarch64|arm64) ARCH_NAME="arm64" ;;
esac
RELEASE_PLATFORM="${RELEASE_PLATFORM:-${OS_NAME}-${ARCH_NAME}}"
RELEASE_DIR="${RELEASE_DIR:-releases/v${RELEASE_VERSION}/${RELEASE_PLATFORM}}"
RELEASE_SIGNING_CERT="${RELEASE_SIGNING_CERT:-}"

[[ -d "${RELEASE_DIR}" ]] || fail "Release directory not found: ${RELEASE_DIR}"
[[ -f "${RELEASE_SIGNING_CERT}" ]] || fail "RELEASE_SIGNING_CERT file is required"

manifest_file="${RELEASE_DIR}/manifest.secure.json"
sha256s="${RELEASE_DIR}/SHA256SUMS"
sha3s="${RELEASE_DIR}/SHA3-512SUMS"
blake3s="${RELEASE_DIR}/BLAKE3SUMS"
for f in "${manifest_file}" "${sha256s}" "${sha3s}" "${blake3s}" \
  "${RELEASE_DIR}/signatures/manifest.secure.json.sig" \
  "${RELEASE_DIR}/signatures/SHA256SUMS.sig" \
  "${RELEASE_DIR}/signatures/SHA3-512SUMS.sig" \
  "${RELEASE_DIR}/signatures/BLAKE3SUMS.sig"; do
  [[ -f "$f" ]] || fail "Missing required release file: ${f}"
done

pubkey_tmp="$(mktemp)"
openssl x509 -in "${RELEASE_SIGNING_CERT}" -pubkey -noout > "${pubkey_tmp}"
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${RELEASE_DIR}/signatures/manifest.secure.json.sig" "${manifest_file}" >/dev/null
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${RELEASE_DIR}/signatures/SHA256SUMS.sig" "${sha256s}" >/dev/null
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${RELEASE_DIR}/signatures/SHA3-512SUMS.sig" "${sha3s}" >/dev/null
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${RELEASE_DIR}/signatures/BLAKE3SUMS.sig" "${blake3s}" >/dev/null
rm -f "${pubkey_tmp}"

python3 - <<'PY' "${manifest_file}" "${sha256s}" "${sha3s}" "${blake3s}" "${RELEASE_DIR}"
import hashlib, json, pathlib, subprocess, sys
manifest_path, sha256_path, sha3_path, b3_path, release_dir = sys.argv[1:6]
release_dir = pathlib.Path(release_dir)
manifest = json.loads(pathlib.Path(manifest_path).read_text(encoding='utf-8'))

def read_sum_file(path):
    m = {}
    for line in pathlib.Path(path).read_text(encoding='utf-8').strip().splitlines():
        parts = line.split()
        if len(parts) < 2:
            raise SystemExit(f'invalid checksum line: {line}')
        m[parts[-1]] = parts[0]
    return m

s256 = read_sum_file(sha256_path)
s3 = read_sum_file(sha3_path)
sb3 = read_sum_file(b3_path)

for art in manifest.get('artifacts', []):
    rel = art['path']
    payload = (release_dir / rel).read_bytes()
    d256 = hashlib.sha256(payload).hexdigest()
    d3 = subprocess.check_output(['openssl', 'dgst', '-sha3-512', str(release_dir / rel)], text=True).split()[-1]
    if s256.get(rel) != d256 or art.get('sha256') != d256:
        raise SystemExit(f'sha256 mismatch: {rel}')
    if s3.get(rel) != d3 or art.get('sha3_512') != d3:
        raise SystemExit(f'sha3-512 mismatch: {rel}')
    if art.get('blake3') != 'unavailable' and sb3.get(rel) != art.get('blake3'):
        raise SystemExit(f'blake3 mismatch: {rel}')
print('secure-bundle verification passed')
PY

log "Secure binary bundle verification passed: ${RELEASE_DIR}"
