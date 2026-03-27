#!/usr/bin/env bash
# AOXC MIT License
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
set -euo pipefail

# ==============================================================================
# AOXChain Production Quality Gate & Security Audit (v1.0.0)
#
# This script enforces strict validation for CI/CD and Pre-release cycles.
# It includes security audits, dependency checks, and comprehensive testing.
# ==============================================================================

MODE="${1:-full}"
DESKTOP_CRATE="aoxchub"

# Requirement verification
require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "[ERROR] Required dependency not found: $cmd" >&2
        echo "Please install it to continue (e.g., cargo install $cmd)" >&2
        exit 127
    fi
}

ensure_cargo_tool() {
    local tool="$1"
    if ! command -v "$tool" >/dev/null 2>&1; then
        printf '\n\033[1;33m==> Installing missing cargo tool: %s\033[0m\n' "$tool"
        cargo install "$tool" --locked
    fi
}

run() {
    local label="$1"
    shift
    printf '\n\033[1;34m==> %s\033[0m\n' "$label"
    if ! "$@"; then
        echo "[FAILURE] Step '$label' failed with exit code $?" >&2
        exit 1
    fi
}

# Ensure core toolchain is present
require_cmd cargo

case "${MODE}" in
    quick)
        run "Format Check" cargo fmt --all --check
        run "Compile Check (Core Workspace)" cargo check --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets
        run "Locked Unit Tests (Core Workspace)" cargo test --locked --workspace --exclude "${DESKTOP_CRATE}" --no-fail-fast
        ;;

    full)
        ensure_cargo_tool cargo-audit
        run "Security Audit" cargo audit
        run "Format Check" cargo fmt --all --check
        run "Linter (Clippy, Core Workspace)" cargo clippy --workspace --exclude "${DESKTOP_CRATE}" --all-targets --all-features -- -D warnings
        run "Compile Check (Core Workspace)" cargo check --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets
        run "Locked Comprehensive Tests (Core Workspace)" cargo test --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets --no-fail-fast
        run "Doc Tests" cargo test --doc
        ;;

    audit | release)
        ensure_cargo_tool cargo-audit
        ensure_cargo_tool cargo-deny
        run "Vulnerability Audit" cargo audit
        run "License & Dependency Audit" cargo deny check
        run "Format Check" cargo fmt --all --check
        run "Compile Check (Release, Core Workspace)" cargo check --locked --workspace --exclude "${DESKTOP_CRATE}" --all-targets --release
        run "Clippy (Strict, Core Workspace)" cargo clippy --workspace --exclude "${DESKTOP_CRATE}" --all-targets --all-features -- -D warnings
        run "Build Production Binary" cargo build --locked --release -p aoxcmd --bin aoxc
        run "Release Artifact Certification" ./scripts/release_artifact_certify.sh target/release/aoxc
        run "Locked Production Test Suite (Core Workspace)" cargo test --locked --workspace --exclude "${DESKTOP_CRATE}" --release --all-targets --no-fail-fast
        ;;

    desktop)
        run "Compile Check (Desktop Tauri Surface)" cargo check --locked -p "${DESKTOP_CRATE}" --all-targets
        ;;

    *)
        echo "Usage: $0 {quick|full|audit|release|desktop}" >&2
        exit 2
        ;;
esac

printf '\n\033[1;32m[SUCCESS] Quality gate passed in "%s" mode.\033[0m\n' "${MODE}"
