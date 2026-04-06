#!/usr/bin/env bash
# ----------------------------------------------------------------------------
# Create a signed release bundle for operator-consumable binaries.
# Required binaries: aoxc, aoxchub, aoxckit
# Required signing inputs: RELEASE_SIGNING_KEY + RELEASE_SIGNING_CERT
# ----------------------------------------------------------------------------

set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

log() {
  printf '[signed-release] %s\n' "$*"
}

fail() {
  printf '[signed-release][error] %s\n' "$*" >&2
  exit 2
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

require_cmd openssl
require_cmd git

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
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

manifest_tmp="$(mktemp)"
checksums_file="${RELEASE_DIR}/SHA256SUMS"
manifest_file="${RELEASE_DIR}/manifest.json"

: > "${checksums_file}"
printf '{\n' > "${manifest_tmp}"
printf '  "release_version": "%s",\n' "${RELEASE_VERSION}" >> "${manifest_tmp}"
printf '  "platform": "%s",\n' "${RELEASE_PLATFORM}" >> "${manifest_tmp}"
printf '  "generated_at_utc": "%s",\n' "$(TZ=UTC date +%Y-%m-%dT%H:%M:%SZ)" >> "${manifest_tmp}"
printf '  "git_commit": "%s",\n' "$(git rev-parse HEAD)" >> "${manifest_tmp}"
printf '  "artifacts": [\n' >> "${manifest_tmp}"

for i in "${!RELEASE_BINARIES[@]}"; do
  bin="${RELEASE_BINARIES[$i]}"
  src="${RELEASE_BIN_DIR}/${bin}"
  [[ -f "${src}" ]] || fail "Missing binary: ${src}"

  dst="${RELEASE_DIR}/bin/${bin}"
  cp "${src}" "${dst}"
  chmod +x "${dst}" || true

  hash="$(sha256_file "${dst}")"
  printf '%s  %s\n' "${hash}" "bin/${bin}" >> "${checksums_file}"

  comma=','
  if [[ "${i}" -eq $((${#RELEASE_BINARIES[@]} - 1)) ]]; then
    comma=''
  fi

  printf '    {"name":"%s","path":"bin/%s","sha256":"%s"}%s\n' "${bin}" "${bin}" "${hash}" "${comma}" >> "${manifest_tmp}"

done

printf '  ]\n}\n' >> "${manifest_tmp}"
mv "${manifest_tmp}" "${manifest_file}"

# Sign core surfaces
openssl dgst -sha256 -sign "${RELEASE_SIGNING_KEY}" -out "${RELEASE_DIR}/signatures/manifest.json.sig" "${manifest_file}"
openssl dgst -sha256 -sign "${RELEASE_SIGNING_KEY}" -out "${RELEASE_DIR}/signatures/SHA256SUMS.sig" "${checksums_file}"

# Verify signatures immediately (fail closed)
pubkey_tmp="$(mktemp)"
openssl x509 -in "${RELEASE_SIGNING_CERT}" -pubkey -noout > "${pubkey_tmp}"
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${RELEASE_DIR}/signatures/manifest.json.sig" "${manifest_file}" >/dev/null
openssl dgst -sha256 -verify "${pubkey_tmp}" -signature "${RELEASE_DIR}/signatures/SHA256SUMS.sig" "${checksums_file}" >/dev/null
rm -f "${pubkey_tmp}"

log "Signed release bundle created: ${RELEASE_DIR}"
log "Artifacts: aoxc, aoxchub, aoxckit"
log "Manifest: ${manifest_file}"
log "Checksums: ${checksums_file}"
