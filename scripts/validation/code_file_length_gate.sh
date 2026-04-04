#!/usr/bin/env bash
set -euo pipefail

MAX_CODE_LINES="${MAX_CODE_LINES:-260}"
MODE="changed"
REPORT_ONLY=0

usage() {
  cat <<USAGE
Usage:
  $(basename "$0") [--all] [--report] [FILE ...]

Options:
  --all      Evaluate all tracked source files instead of changed files.
  --report   Print violating files and exit 0 (report-only mode).
  -h, --help Show this help.

Environment:
  MAX_CODE_LINES   Maximum allowed lines per source file (default: 260).
USAGE
}

if ! [[ "$MAX_CODE_LINES" =~ ^[0-9]+$ ]]; then
  echo "MAX_CODE_LINES must be a positive integer. Got: $MAX_CODE_LINES" >&2
  exit 1
fi

CODE_EXTENSIONS_REGEX='\.(rs|py|ts|tsx|js|jsx|sol|go|c|cc|cpp|h|hpp|java|kt|swift|rb|php|cs|sh)$'

FILES_FROM_ARGS=()
while [ "$#" -gt 0 ]; do
  case "$1" in
    --all)
      MODE="all"
      ;;
    --report)
      REPORT_ONLY=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      FILES_FROM_ARGS+=("$1")
      ;;
  esac
  shift
done

collect_files() {
  if [ "${#FILES_FROM_ARGS[@]}" -gt 0 ]; then
    printf '%s\n' "${FILES_FROM_ARGS[@]}"
    return
  fi

  if [ "$MODE" = "all" ]; then
    git ls-files
    return
  fi

  local staged
  staged="$(git diff --cached --name-only --diff-filter=ACMR)"
  if [ -n "$staged" ]; then
    printf '%s\n' "$staged"
    return
  fi

  local modified
  modified="$(git diff --name-only --diff-filter=ACMR)"
  if [ -n "$modified" ]; then
    printf '%s\n' "$modified"
    return
  fi

  echo ""
}

mapfile -t CANDIDATES < <(collect_files)

if [ "${#CANDIDATES[@]}" -eq 0 ] || { [ "${#CANDIDATES[@]}" -eq 1 ] && [ -z "${CANDIDATES[0]}" ]; }; then
  echo "Code file length gate skipped: no files selected."
  exit 0
fi

checked=0
violations=0
for file in "${CANDIDATES[@]}"; do
  [ -f "$file" ] || continue

  if [[ ! "$file" =~ $CODE_EXTENSIONS_REGEX ]]; then
    continue
  fi

  checked=$((checked + 1))
  line_count="$(wc -l < "$file" | tr -d ' ')"
  if [ "$line_count" -gt "$MAX_CODE_LINES" ]; then
    echo "[FAIL] $file has $line_count lines (max: $MAX_CODE_LINES)."
    violations=$((violations + 1))
  fi
done

if [ "$checked" -eq 0 ]; then
  echo "Code file length gate skipped: no source files matched configured extensions."
  exit 0
fi

if [ "$violations" -gt 0 ]; then
  if [ "$REPORT_ONLY" -eq 1 ]; then
    echo "Code file length report completed: $violations violation(s), $checked source file(s) checked."
    exit 0
  fi
  echo "Code file length gate failed with $violations violation(s), $checked source file(s) checked."
  exit 1
fi

echo "Code file length gate passed: 0 violations, $checked source file(s) checked (max: $MAX_CODE_LINES lines)."
