#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CONFIG="$ROOT/backend/app/tauri.conf.json"

cd "$ROOT/frontend"
npm install

npm run dev &
FRONTEND_PID=$!
trap 'kill "$FRONTEND_PID" 2>/dev/null || true' EXIT INT TERM

for _ in $(seq 1 30); do
  if curl -sf "http://localhost:5173" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

cd "$ROOT/backend/app"
if command -v cargo-tauri >/dev/null 2>&1; then
  cargo tauri dev --config "$CONFIG"
else
  npx --yes @tauri-apps/cli@2 dev --config "$CONFIG"
fi
