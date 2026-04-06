#!/usr/bin/env bash
# ----------------------------------------------------------------------------
# Repository hygiene gate
# - blocks accidental commits of local virtualenvs
# - blocks generated release bundles under releases/v*/
# - blocks common compiled binary/object artifacts
# ----------------------------------------------------------------------------

set -euo pipefail
IFS=$'\n\t'

log() {
  printf '[repo-hygiene] %s\n' "$*"
}

fail() {
  printf '[repo-hygiene][error] %s\n' "$*" >&2
  exit 2
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

tracked_files="$(git ls-files)"

if [[ -z "${tracked_files}" ]]; then
  fail "Unable to resolve tracked files from git ls-files."
fi

forbidden_path_regex='(^|/)\.venv/|(^|/)venv/|^releases/v[^/]+/(binaries/|signatures/|manifest\.json$|checksums\.sha256$|compatibility\.toml$|sbom\.spdx\.json$|provenance\.intoto\.jsonl$)|\.(exe|dll|so|dylib|o|obj|a)$'

forbidden_paths="$(printf '%s\n' "${tracked_files}" | grep -E "${forbidden_path_regex}" || true)"
if [[ -n "${forbidden_paths}" ]]; then
  printf '[repo-hygiene][error] Forbidden tracked paths detected:\n%s\n' "${forbidden_paths}" >&2
  fail "Remove generated/local/binary artifacts from git tracking."
fi

max_bytes=$((5 * 1024 * 1024))
large_files=()
large_file_allowlist_regex='^logos/'
while IFS= read -r rel_path; do
  [[ -z "${rel_path}" ]] && continue
  if [[ -f "${rel_path}" ]]; then
    size_bytes=$(wc -c < "${rel_path}" | tr -d '[:space:]')
    if [[ "${size_bytes}" -gt "${max_bytes}" ]] && [[ ! "${rel_path}" =~ ${large_file_allowlist_regex} ]]; then
      large_files+=("${rel_path} (${size_bytes} bytes)")
    fi
  fi
done <<< "${tracked_files}"

if [[ "${#large_files[@]}" -gt 0 ]]; then
  printf '[repo-hygiene][error] Oversized tracked files (>5MB) detected:\n' >&2
  printf ' - %s\n' "${large_files[@]}" >&2
  fail "Move large generated artifacts to release storage/artifacts instead of git."
fi

log "Repository hygiene gate passed."
