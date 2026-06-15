# Contributing to OCorganize

## Prerequisites

- Rust stable
- Node.js 22+
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

## Development

```bash
npm install
npm run dev
```

## Test

```bash
npm run test
```

Runs `cargo test --workspace` and frontend type checks.

## Project layout

- `backend/core/` — filesystem logic (walk, group, move, undo)
- `backend/app/` — Tauri commands and job orchestration
- `frontend/` — Svelte 5 UI
- `docs/CONTEXT.md` — domain terminology

## Domain language

Use terms from `docs/CONTEXT.md`. The app performs **LooseRootScan**: only loose files directly in the organization root are organized. Do not describe the scanner as recursive unless that behavior is intentionally changed.

## Pull requests

- Keep changes focused.
- Run `npm run test` before opening a PR.
- Update `docs/CONTEXT.md` or ADRs when behavior or terminology changes.
