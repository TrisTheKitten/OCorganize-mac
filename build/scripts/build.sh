#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT/frontend"
npm ci
bash "$ROOT/build/scripts/frontend-build.sh"

cd "$ROOT/backend/app"
chmod +x dev-frontend.sh build-frontend.sh

if [ ! -f icons/icon.icns ] && [ -f app-icon.png ]; then
  if command -v cargo-tauri >/dev/null 2>&1; then
    cargo tauri icon app-icon.png -o icons
  else
    npx --yes @tauri-apps/cli@2 icon app-icon.png -o icons
  fi
fi

if command -v cargo-tauri >/dev/null 2>&1; then
  cargo tauri build --config "$ROOT/backend/app/tauri.conf.json"
else
  npx --yes @tauri-apps/cli@2 build --config "$ROOT/backend/app/tauri.conf.json"
fi
