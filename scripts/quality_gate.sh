#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# AOXChain Production Quality Gate & Security Audit (v1.0.0)
#
# This script enforces strict validation for CI/CD and Pre-release cycles.
# It includes security audits, dependency checks, and comprehensive testing.
# ==============================================================================

MODE="${1:-full}"

# Requirement verification
require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "[ERROR] Required dependency not found: $cmd" >&2
        echo "Please install it to continue (e.g., cargo install $cmd)" >&2
        exit 127
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
        run "Format Check" cargo fmt --all -- --check
        run "Compile Check" cargo check --workspace
        run "Locked Unit Tests" cargo test --locked --workspace --no-fail-fast
        ;;

    full)
        require_cmd cargo-audit
        run "Security Audit" cargo audit
        run "Format Check" cargo fmt --all -- --check
        run "Linter (Clippy)" cargo clippy --workspace --all-targets --all-features -- -D warnings
        run "Compile Check" cargo check --workspace --all-targets
        run "Locked Comprehensive Tests" cargo test --locked --workspace --all-targets --no-fail-fast
        run "Doc Tests" cargo test --doc
        ;;

    audit | release)
        require_cmd cargo-audit
        require_cmd cargo-deny
        run "Vulnerability Audit" cargo audit
        run "License & Dependency Audit" cargo deny check
        run "Format Check" cargo fmt --all -- --check
        run "Compile Check (Release)" cargo check --workspace --release
        run "Clippy (Strict)" cargo clippy --workspace --all-targets --all-features -- -D warnings
        run "Build Production Binary" cargo build --release -p aoxcmd --bin aoxc
        run "Release Artifact Certification" ./scripts/release_artifact_certify.sh target/release/aoxc
        run "Locked Production Test Suite" cargo test --locked --workspace --release --all-targets --no-fail-fast
        ;;

    *)
        echo "Usage: $0 {quick|full|audit|release}" >&2
        exit 2
        ;;
esac

printf '\n\033[1;32m[SUCCESS] Quality gate passed in "%s" mode.\033[0m\n' "${MODE}"
