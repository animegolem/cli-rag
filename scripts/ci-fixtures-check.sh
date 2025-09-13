#!/usr/bin/env bash
set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for ci-fixtures-check. Please install jq." >&2
  exit 1
fi

# Build debug binary for speed
cargo build -q
BIN=./target/debug/cli-rag

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT
NOTES="$WORKDIR/notes"
mkdir -p "$NOTES"
CFG="$WORKDIR/.cli-rag.toml"

# Minimal config
cat > "$CFG" <<EOF
bases = [
  '$NOTES'
]
EOF

# Add schema mapping
cat >> "$CFG" <<'EOF'

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"
EOF

# 1) info / validate dry-run smoke
$BIN --config "$CFG" info --format json | jq -e '.protocolVersion >= 1' >/dev/null
$BIN --config "$CFG" validate --format json --dry-run | jq -e '.ok == true and .docCount == 0' >/dev/null

# 2) search (note)
cat > "$NOTES/ADR-100.md" <<'MD'
---
id: ADR-100
tags: [x]
status: draft
depends_on: []
---

# ADR-100: Title
MD
$BIN --config "$CFG" validate --format json >/dev/null
OUT=$($BIN --config "$CFG" search -q ADR-100 --format json)
echo "$OUT" | jq -e '.results | any(.kind=="note")' >/dev/null

# 3) search (todo + kanban)
cat > "$NOTES/ADR-101.md" <<'MD'
---
id: ADR-101
tags: [x]
status: draft
depends_on: []
kanban_status: doing
kanban_statusline: In progress
due_date: 2026-01-01
---

# ADR-101: Title

- [ ] Task one
MD
$BIN --config "$CFG" validate --format json >/dev/null
OUT=$($BIN --config "$CFG" search -q ADR-101 --kind kanban,todo --format json)
echo "$OUT" | jq -e '.results | any(.kind=="todo")' >/dev/null
echo "$OUT" | jq -e '.results | any(.kind=="kanban")' >/dev/null

# 4) ai get JSON
cat > "$NOTES/ADR-150.md" <<'MD'
---
id: ADR-150
tags: [x]
status: draft
depends_on: []
---

# ADR-150: Title

Body
MD
$BIN --config "$CFG" validate --format json >/dev/null
OUT=$($BIN --config "$CFG" get --id ADR-150 --format json)
echo "$OUT" | jq -e '.protocolVersion==1 and .retrievalVersion==1' >/dev/null

echo "local ci fixtures OK"
