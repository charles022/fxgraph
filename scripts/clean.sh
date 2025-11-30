#!/usr/bin/env bash
set -euo pipefail

# Remove compiled and generated artifacts for a fresh workspace.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DRY_RUN=false

if [[ "${1:-}" =~ ^(--dry-run|-n)$ ]]; then
  DRY_RUN=true
  shift
fi

if [[ $# -gt 0 ]]; then
  echo "Usage: $(basename "$0") [--dry-run]" >&2
  exit 1
fi

CLEAN_DIRS=(
  "$ROOT_DIR/backend/target"          # Rust build output
  "$ROOT_DIR/frontend/node_modules"   # Frontend dependencies
  "$ROOT_DIR/frontend/.next"          # Next.js build output
  "$ROOT_DIR/frontend/.turbo"         # Turbopack cache
  "$ROOT_DIR/frontend/.swc"           # SWC cache
  "$ROOT_DIR/frontend/out"            # Next.js export output
  "$ROOT_DIR/frontend/.buf-cache"     # Buf download/cache dir
  "$ROOT_DIR/frontend/gen"            # Generated TS protobuf/client code
)

CLEAN_FILES=(
  "$ROOT_DIR/frontend/tsconfig.tsbuildinfo" # TypeScript incremental cache
  "$ROOT_DIR/frontend/.eslintcache"         # ESLint cache
)

remove_path() {
  local path="$1"
  [[ -e "$path" ]] || return 0

  if [[ "$DRY_RUN" == true ]]; then
    echo "[dry-run] rm -rf \"$path\""
  else
    rm -rf -- "$path"
    echo "Removed: $path"
  fi
}

for path in "${CLEAN_DIRS[@]}" "${CLEAN_FILES[@]}"; do
  remove_path "$path"
done

if [[ "$DRY_RUN" == true ]]; then
  echo "Dry run complete. No files were removed."
else
  echo "Workspace clean complete."
fi
