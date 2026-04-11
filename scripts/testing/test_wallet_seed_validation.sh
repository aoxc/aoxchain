#!/usr/bin/env bash
set -Eeuo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
script_path="${repo_root}/scripts/wallet_seed.sh"

tmp_root="$(mktemp -d)"
cleanup() {
  rm -rf "${tmp_root}"
}
trap cleanup EXIT

mkdir -p "${tmp_root}/bin"
cat > "${tmp_root}/bin/make" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" > "${TMP_MAKE_ARGS_PATH:?missing TMP_MAKE_ARGS_PATH}"
MOCK
chmod +x "${tmp_root}/bin/make"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    printf 'assertion failed: expected to find "%s" in "%s"\n' "${needle}" "${haystack}" >&2
    exit 1
  fi
}

assert_fails_with() {
  local expected="$1"
  shift
  local output
  if output="$("$@" 2>&1)"; then
    printf 'assertion failed: command unexpectedly succeeded: %s\n' "$*" >&2
    exit 1
  fi
  assert_contains "${output}" "${expected}"
}

make_args_file="${tmp_root}/make.args"
export TMP_MAKE_ARGS_PATH="${make_args_file}"

PATH="${tmp_root}/bin:${PATH}" \
AOXC_NEW_ACCOUNT_ID="AOXC_USER_BETA_01" \
AOXC_NEW_ACCOUNT_BALANCE="12345" \
AOXC_NEW_ACCOUNT_ROLE="USER" \
  "${script_path}"

make_args="$(cat "${make_args_file}")"
assert_contains "${make_args}" "chain-add-account"
assert_contains "${make_args}" "AOXC_NEW_ACCOUNT_ROLE=user"

assert_fails_with "AOXC_NEW_ACCOUNT_ID must match" \
  env PATH="${tmp_root}/bin:${PATH}" AOXC_NEW_ACCOUNT_ID="bad id" "${script_path}"

assert_fails_with "AOXC_NEW_ACCOUNT_ROLE 'bridge' is unsupported" \
  env PATH="${tmp_root}/bin:${PATH}" AOXC_NEW_ACCOUNT_ROLE="bridge" "${script_path}"

assert_fails_with "AOXC_NEW_ACCOUNT_BALANCE must be greater than zero" \
  env PATH="${tmp_root}/bin:${PATH}" AOXC_NEW_ACCOUNT_BALANCE="0" "${script_path}"

PATH="${tmp_root}/bin:${PATH}" \
AOXC_NEW_ACCOUNT_BALANCE="0" \
AOXC_ALLOW_ZERO_BALANCE="1" \
  "${script_path}"

