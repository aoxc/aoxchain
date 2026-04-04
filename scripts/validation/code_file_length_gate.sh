#!/usr/bin/env bash
set -euo pipefail

MAX_CODE_LINES="${MAX_CODE_LINES:-260}"
SCAN_MODE="changed"

CODE_EXTENSIONS_REGEX='\.(rs|py|ts|tsx|js|jsx|sol|go|c|cc|cpp|h|hpp|java|kt|swift|rb|php|cs|sh)$'

usage() {
  cat <<'USAGE'
Usage: code_file_length_gate.sh [options] [files...]

Options:
  --all              Evaluate all tracked code files in the repository.
  --changed          Evaluate only staged/modified files (default).
  --max-lines <N>    Override the maximum line count threshold.
  -h, --help         Show this help text.

Environment:
  MAX_CODE_LINES     Default line threshold when --max-lines is not provided.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --all)
      SCAN_MODE="all"
      shift
      ;;
    --changed)
      SCAN_MODE="changed"
      shift
      ;;
    --max-lines)
      if [ "$#" -lt 2 ]; then
        echo "--max-lines requires a numeric value." >&2
        exit 1
      fi
      MAX_CODE_LINES="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
    *)
      break
      ;;
  esac
done

if ! [[ "$MAX_CODE_LINES" =~ ^[0-9]+$ ]] || [ "$MAX_CODE_LINES" -le 0 ]; then
  echo "MAX_CODE_LINES must be a positive integer. Got: $MAX_CODE_LINES" >&2
  exit 1
fi

collect_files() {
  if [ "$#" -gt 0 ]; then
    printf '%s\n' "$@"
    return
  fi

  if [ "$SCAN_MODE" = "all" ]; then
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

  echo "No staged or modified files to evaluate."
}

mapfile -t CANDIDATES < <(collect_files "$@")

if [ "${#CANDIDATES[@]}" -eq 1 ] && [ "${CANDIDATES[0]}" = "No staged or modified files to evaluate." ]; then
  echo "Code file length gate skipped: no changed files."
  exit 0
fi

violations=0
evaluated=0
for file in "${CANDIDATES[@]}"; do
  [ -f "$file" ] || continue

  if [[ ! "$file" =~ $CODE_EXTENSIONS_REGEX ]]; then
    continue
  fi

  evaluated=$((evaluated + 1))
  line_count="$(wc -l < "$file" | tr -d ' ')"
  if [ "$line_count" -gt "$MAX_CODE_LINES" ]; then
    echo "[FAIL] $file has $line_count lines (max: $MAX_CODE_LINES)."
    violations=$((violations + 1))
  fi
done

if [ "$violations" -gt 0 ]; then
  echo "Code file length gate failed with $violations violation(s) across $evaluated file(s)."
  exit 1
fi

echo "Code file length gate passed (max: $MAX_CODE_LINES lines, evaluated: $evaluated file(s))."
