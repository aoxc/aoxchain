#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

fail() {
  echo "[versioning-gate] ERROR: $*" >&2
  exit 1
}

log() {
  echo "[versioning-gate] $*"
}

extract_toml_value() {
  local file="$1"
  local key="$2"
  sed -n "s/^${key} = \"\(.*\)\"/\1/p" "$file" | head -n1
}

read_workspace_version() {
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' Cargo.toml | head -n1
}

read_policy_version() {
  sed -n '/^\[workspace\]/,/^\[/{s/^current = "\(.*\)"/\1/p}' configs/version-policy.toml | head -n1
}

version_to_triplet() {
  local version="$1"
  local core
  core="${version%%-*}"
  IFS='.' read -r major minor patch <<<"${core}"
  [[ -n "${major:-}" && -n "${minor:-}" && -n "${patch:-}" ]] || return 1
  printf '%d %d %d\n' "$major" "$minor" "$patch"
}

version_gt() {
  local left="$1"
  local right="$2"
  local l_major l_minor l_patch r_major r_minor r_patch
  read -r l_major l_minor l_patch < <(version_to_triplet "$left")
  read -r r_major r_minor r_patch < <(version_to_triplet "$right")

  if (( l_major > r_major )); then return 0; fi
  if (( l_major < r_major )); then return 1; fi
  if (( l_minor > r_minor )); then return 0; fi
  if (( l_minor < r_minor )); then return 1; fi
  if (( l_patch > r_patch )); then return 0; fi
  return 1
}

version="$({ read_workspace_version; } || true)"
policy_version="$({ read_policy_version; } || true)"
strategy="$(extract_toml_value configs/version-policy.toml strategy)"
release_channel="$(extract_toml_value configs/version-policy.toml release_channel)"

[[ -n "$version" ]] || fail "Cannot resolve [workspace.package].version from Cargo.toml"
[[ -n "$policy_version" ]] || fail "Cannot resolve [workspace].current from configs/version-policy.toml"
[[ "$version" == "$policy_version" ]] || fail "Version mismatch: Cargo.toml=$version, version-policy=$policy_version"
[[ "$strategy" == "global-workspace-version-with-component-schema-tracks" ]] || fail "Unsupported version strategy: $strategy"
[[ "$release_channel" == "controlled" ]] || fail "Unsupported release channel: $release_channel"

if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  fail "Workspace version '$version' must satisfy SemVer core format MAJOR.MINOR.PATCH[-PRERELEASE]"
fi

head_tag="$(git tag --points-at HEAD | grep -E '^v' | head -n1 || true)"
if [[ -n "$head_tag" && "$head_tag" != "v${version}" ]]; then
  fail "HEAD tag '$head_tag' must match workspace version tag 'v${version}'"
fi

base_ref="${AOXC_VERSION_BASE_REF:-}"
if [[ -z "$base_ref" ]]; then
  if git rev-parse --verify --quiet HEAD~1 >/dev/null; then
    base_ref="HEAD~1"
  else
    base_ref=""
  fi
fi

if [[ -n "$base_ref" ]]; then
  changed_files="$({
    git diff --name-only "$base_ref"...HEAD
    git diff --name-only
    git diff --cached --name-only
  } | awk 'NF' | sort -u)"

  if [[ -n "$changed_files" ]]; then
    relevant_changes="$(printf '%s\n' "$changed_files" | grep -E '^(crates/|contracts/|configs/|scripts/|Cargo.toml|Cargo.lock|Makefile|Dockerfile|tests/)')"
    if [[ -n "$relevant_changes" ]]; then
      if ! printf '%s\n' "$changed_files" | grep -Eq '^Cargo.toml$|^configs/version-policy.toml$'; then
        fail "Version-sensitive changes detected since ${base_ref}, but neither Cargo.toml nor configs/version-policy.toml was updated"
      fi
    fi
  fi
fi

latest_tag="$(git tag -l 'v*' --sort=-v:refname | head -n1 || true)"
if [[ -n "$latest_tag" ]]; then
  latest_version="${latest_tag#v}"
  if ! version_gt "$version" "$latest_version"; then
    if [[ "${AOXC_ALLOW_NON_INCREMENTAL_VERSION:-0}" != "1" ]]; then
      fail "Workspace version '$version' must be greater than latest git tag '$latest_tag' (set AOXC_ALLOW_NON_INCREMENTAL_VERSION=1 to override)"
    fi
    log "Non-incremental version accepted by override against latest tag ${latest_tag}"
  fi
fi

log "Version governance checks passed for ${version}"
