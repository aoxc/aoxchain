#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# AOXC MIT License
#
# Experimental software under active construction.
# This file is part of the AOXC pre-release codebase.
# -----------------------------------------------------------------------------
#
# Compatibility shim:
#   Keep legacy script path operational while delegating implementation to the
#   canonical release surface at scripts/release/generate_release_evidence.sh.
# -----------------------------------------------------------------------------

set -Eeuo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "${SCRIPT_DIR}/release/generate_release_evidence.sh" "$@"
