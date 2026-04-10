#!/usr/bin/env bash
# AOXC MIT License
# Download and install release binaries from GitHub release assets with verification.

set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

log() { printf '[github-install] %s\n' "$*"; }
fail() { printf '[github-install][error] %s\n' "$*" >&2; exit 2; }
require_cmd() { command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"; }

usage() {
  cat <<USAGE
Usage:
  ./scripts/release/github_binary_install.sh --repo owner/repo --version 0.2.0-alpha.3 [options]

Options:
  --repo <owner/repo>          GitHub repository (required)
  --version <semver>           Release version without v prefix (required)
  --platform <os-arch>         Platform suffix (default: auto detect)
  --install-dir <path>         Install target directory. Default: <AOXC_ROOT>/bin/current
  --downloads-dir <path>       Download cache directory. Default: <AOXC_ROOT>/downloads/github/v<version>/<platform>
  --cert <path>                Optional certificate for signature verification
  --github-base-url <url>      Override base URL (default: https://github.com)
USAGE
}

require_cmd curl
require_cmd tar
require_cmd python3

REPO=""
VERSION=""
PLATFORM=""
INSTALL_DIR=""
DOWNLOADS_DIR=""
CERT_PATH=""
GITHUB_BASE_URL="https://github.com"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo) REPO="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --platform) PLATFORM="$2"; shift 2 ;;
    --install-dir) INSTALL_DIR="$2"; shift 2 ;;
    --downloads-dir) DOWNLOADS_DIR="$2"; shift 2 ;;
    --cert) CERT_PATH="$2"; shift 2 ;;
    --github-base-url) GITHUB_BASE_URL="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Unknown argument: $1" ;;
  esac
done

[[ -n "$REPO" ]] || fail "--repo is required"
[[ -n "$VERSION" ]] || fail "--version is required"

if [[ -z "$PLATFORM" ]]; then
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m | tr '[:upper:]' '[:lower:]')"
  case "${arch}" in
    x86_64|amd64) arch="amd64" ;;
    aarch64|arm64) arch="arm64" ;;
  esac
  PLATFORM="${os}-${arch}"
fi

if [[ -z "${AOXC_ROOT:-}" ]]; then
  if [[ -n "${XDG_STATE_HOME:-}" ]]; then
    AOXC_ROOT="${XDG_STATE_HOME}/aoxc"
  elif [[ -n "${XDG_DATA_HOME:-}" ]]; then
    AOXC_ROOT="${XDG_DATA_HOME}/aoxc"
  else
    AOXC_ROOT="${HOME}/.aoxc"
  fi
fi

INSTALL_DIR="${INSTALL_DIR:-${AOXC_ROOT}/bin/current}"
DOWNLOADS_DIR="${DOWNLOADS_DIR:-${AOXC_ROOT}/downloads/github/v${VERSION}/${PLATFORM}}"
mkdir -p "${INSTALL_DIR}" "${DOWNLOADS_DIR}"

archive_name="aoxc-v${VERSION}-${PLATFORM}-portable.tar.gz"
checksums_name="SHA256SUMS"
sig_name="SHA256SUMS.sig"

base="${GITHUB_BASE_URL}/${REPO}/releases/download/v${VERSION}"
archive_path="${DOWNLOADS_DIR}/${archive_name}"
checksums_path="${DOWNLOADS_DIR}/${checksums_name}"
sig_path="${DOWNLOADS_DIR}/${sig_name}"

log "Downloading release assets from ${base}"
curl -fL "${base}/${archive_name}" -o "${archive_path}"
curl -fL "${base}/${checksums_name}" -o "${checksums_path}"
if curl -fL "${base}/${sig_name}" -o "${sig_path}"; then
  log "Fetched signature file ${sig_name}"
else
  log "Signature file not found; continuing with checksum verification only"
  rm -f "${sig_path}"
fi

python3 - <<'PY' "${archive_path}" "${checksums_path}"
import hashlib, pathlib, sys
archive, sums = map(pathlib.Path, sys.argv[1:3])
name = archive.name
expected = None
for line in sums.read_text(encoding='utf-8').splitlines():
    parts = line.split()
    if len(parts) < 2:
        continue
    if parts[-1].endswith(name) or parts[-1] == name:
        expected = parts[0]
        break
if not expected:
    raise SystemExit(f'missing checksum entry for {name}')
actual = hashlib.sha256(archive.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit(f'checksum mismatch for {name}: expected={expected} actual={actual}')
print('archive checksum verification passed')
PY

if [[ -n "$CERT_PATH" && -f "$sig_path" ]]; then
  require_cmd openssl
  pubkey_tmp="$(mktemp)"
  openssl x509 -in "$CERT_PATH" -pubkey -noout > "$pubkey_tmp"
  openssl dgst -sha256 -verify "$pubkey_tmp" -signature "$sig_path" "$checksums_path" >/dev/null
  rm -f "$pubkey_tmp"
  log "Certificate-based signature verification passed"
fi

extract_dir="${DOWNLOADS_DIR}/extract"
rm -rf "$extract_dir"
mkdir -p "$extract_dir"
tar -xzf "$archive_path" -C "$extract_dir"

for bin in aoxc aoxchub aoxckit; do
  src="$(find "$extract_dir" -type f -name "$bin" | head -n1 || true)"
  [[ -n "$src" ]] || fail "Missing binary in archive: ${bin}"
  cp "$src" "${INSTALL_DIR}/${bin}"
  chmod +x "${INSTALL_DIR}/${bin}" || true
done

log "Installed binaries to ${INSTALL_DIR}"
log "Downloads retained in ${DOWNLOADS_DIR}"
