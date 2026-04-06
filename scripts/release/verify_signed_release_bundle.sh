#!/usr/bin/env bash
# ----------------------------------------------------------------------------
# Verify signed release bundle integrity.
# ----------------------------------------------------------------------------

set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

log() {
  printf '[verify-release] %s\n' "$*"
}

fail() {
  printf '[verify-release][error] %s\n' "$*" >&2
  exit 2
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

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

manifest_file="${RELEASE_DIR}/manifest.json"
checksums_file="${RELEASE_DIR}/SHA256SUMS"
manifest_sig="${RELEASE_DIR}/signatures/manifest.json.sig"
checksums_sig="${RELEASE_DIR}/signatures/SHA256SUMS.sig"

for f in "${manifest_file}" "${checksums_file}" "${manifest_sig}" "${checksums_sig}"; do
  [[ -f "${f}" ]] || fail "Missing required release file: ${f}"
done

pubkey_tmp="$(mktemp)"
openssl x509 -in "${RELEASE_SIGNING_CERT}" -pubkey -noout > "${pubkey_tmp}"
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${manifest_sig}" "${manifest_file}" >/dev/null
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${checksums_sig}" "${checksums_file}" >/dev/null
rm -f "${pubkey_tmp}"

python3 - <<'PY' "${RELEASE_DIR}"
import hashlib
import json
import pathlib
import sys

release_dir = pathlib.Path(sys.argv[1])
manifest = json.loads((release_dir / 'manifest.json').read_text(encoding='utf-8'))

checksum_map = {}
for line in (release_dir / 'SHA256SUMS').read_text(encoding='utf-8').strip().splitlines():
    parts = line.split()
    if len(parts) < 2:
        raise SystemExit(f'invalid checksum line: {line}')
    checksum_map[parts[-1]] = parts[0]

for art in manifest.get('artifacts', []):
    rel = art['path']
    p = release_dir / rel
    if not p.exists():
        raise SystemExit(f'missing artifact: {rel}')
    d = hashlib.sha256(p.read_bytes()).hexdigest()
    if d != art['sha256']:
        raise SystemExit(f'manifest hash mismatch: {rel}')
    if checksum_map.get(rel) != d:
        raise SystemExit(f'SHA256SUMS mismatch: {rel}')

print('release hash verification passed')
PY

log "Signature and checksum verification passed: ${RELEASE_DIR}"
