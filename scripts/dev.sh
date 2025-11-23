#!/usr/bin/env bash
set -euo pipefail

# Root of the repo
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND_DIR="$ROOT_DIR/backend"
FRONTEND_DIR="$ROOT_DIR/frontend"

BACKEND_PORT=50051
FRONTEND_PORT=3000

cleanup() {
  echo "Stopping processes..."
  [[ -n "${BACKEND_PID:-}" ]] && kill "$BACKEND_PID" >/dev/null 2>&1 || true
  [[ -n "${FRONTEND_PID:-}" ]] && kill "$FRONTEND_PID" >/dev/null 2>&1 || true
}
trap cleanup INT TERM EXIT

echo "==> Starting backend on port ${BACKEND_PORT}..."
(
  cd "$BACKEND_DIR"
  cargo run
) &
BACKEND_PID=$!

echo "==> Installing frontend deps and starting dev server on port ${FRONTEND_PORT}..."
(
  cd "$FRONTEND_DIR"
  if [[ ! -d node_modules ]]; then
    npm install
  fi
  npm run generate
  npm run dev -- --hostname 0.0.0.0 --port "${FRONTEND_PORT}"
) &
FRONTEND_PID=$!

echo "Backend PID: ${BACKEND_PID}"
echo "Frontend PID: ${FRONTEND_PID}"
echo "Open http://localhost:${FRONTEND_PORT} once both are ready."
echo "Press Ctrl+C to stop both."

wait
