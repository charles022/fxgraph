#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEX_CMD=${CODEX_CMD:-codex}

for dir in "$root"/*; do
  [ -d "$dir" ] || continue
  readme="$dir/README.md"
  if [ ! -f "$readme" ]; then
    echo "[skip] $(basename "$dir") (no README.md)" >&2
    continue
  fi
  echo "=== Running codex for $(basename "$dir") ===" >&2
  (
    cd "$dir"
    "$CODEX_CMD" "Use README.md in this directory to create a working proof of concept here."
  ) || true
done
