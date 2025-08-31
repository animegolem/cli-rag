#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

MAX=350
FAIL=0

while IFS= read -r -d '' file; do
  # Exclude generated or special cases here if needed
  # Count lines
  LINES=$(wc -l < "$file" | tr -d ' ')
  if [[ "$LINES" -gt "$MAX" ]]; then
    echo "[line-guard] $file has $LINES lines (> $MAX)"
    FAIL=1
  fi
done < <(find src -type f -name "*.rs" -print0)

if [[ "$FAIL" -ne 0 ]]; then
  echo "[line-guard] Consider refactoring files listed above (threshold: $MAX)." >&2
  exit 1
fi

echo "[line-guard] All source files within threshold ($MAX lines)."

