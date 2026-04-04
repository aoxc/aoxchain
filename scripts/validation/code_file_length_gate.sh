#!/usr/bin/env bash
set -euo pipefail

MAX_CODE_LINES="${MAX_CODE_LINES:-260}"

if ! [[ "$MAX_CODE_LINES" =~ ^[0-9]+$ ]]; then
  echo "MAX_CODE_LINES must be a positive integer. Got: $MAX_CODE_LINES" >&2
  exit 1
fi

CODE_EXTENSIONS_REGEX='\.(rs|py|ts|tsx|js|jsx|sol|go|c|cc|cpp|h|hpp|java|kt|swift|rb|php|cs|sh)$'

collect_files() {
  if [ "$#" -gt 0 ]; then
    printf '%s\n' "$@"
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

  echo "No staged or modified files to evaluate."
}

mapfile -t CANDIDATES < <(collect_files "$@")

if [ "${#CANDIDATES[@]}" -eq 1 ] && [ "${CANDIDATES[0]}" = "No staged or modified files to evaluate." ]; then
  echo "Code file length gate skipped: no changed files."
  exit 0
fi

violations=0
for file in "${CANDIDATES[@]}"; do
  [ -f "$file" ] || continue

  if [[ ! "$file" =~ $CODE_EXTENSIONS_REGEX ]]; then
    continue
  fi

  line_count="$(wc -l < "$file" | tr -d ' ')"
  if [ "$line_count" -gt "$MAX_CODE_LINES" ]; then
    echo "[FAIL] $file has $line_count lines (max: $MAX_CODE_LINES)."
    violations=$((violations + 1))
  fi
done

if [ "$violations" -gt 0 ]; then
  echo "Code file length gate failed with $violations violation(s)."
  exit 1
fi

echo "Code file length gate passed (max: $MAX_CODE_LINES lines)."
