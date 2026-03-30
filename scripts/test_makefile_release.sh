#!/usr/bin/env bash
set -euo pipefail

RUN_BUILD=0
RUN_WORKSPACE_TESTS=0

for arg in "$@"; do
  case "$arg" in
    --run-build)
      RUN_BUILD=1
      ;;
    --run-workspace-tests)
      RUN_WORKSPACE_TESTS=1
      ;;
    *)
      echo "Unknown argument: $arg" >&2
      echo "Usage: $0 [--run-build] [--run-workspace-tests]" >&2
      exit 1
      ;;
  esac
done

echo "==> AOXC Makefile release packaging verification"
echo "Repository root: $(pwd)"

echo
echo "==> Step 1: Resolve path model"
make paths

echo
echo "==> Step 2: Detect workspace binaries"
make release-binary-list

echo
echo "==> Step 3: Validate packaging command plans (dry-run)"
make -n package-all-bin
make -n package-versioned-bin
make -n package-versioned-archive

if [[ "$RUN_BUILD" -eq 1 ]]; then
  echo
  echo "==> Step 4: Execute real packaging build"
  make package-versioned-bin
  make package-versioned-archive
fi

if [[ "$RUN_WORKSPACE_TESTS" -eq 1 ]]; then
  echo
  echo "==> Step 5: Pre-check native dependency for workspace tests"
  if ! command -v pkg-config >/dev/null 2>&1; then
    echo "ERROR: pkg-config not found. Install pkg-config first." >&2
    exit 2
  fi

  if ! pkg-config --exists glib-2.0; then
    echo "ERROR: glib-2.0 development package is not available." >&2
    echo "Hint (Debian/Ubuntu): sudo apt-get install -y libglib2.0-dev pkg-config" >&2
    exit 3
  fi

  echo "Running full workspace tests..."
  make test
fi

echo
echo "Verification flow completed."
