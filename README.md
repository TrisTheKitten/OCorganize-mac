# OCorganize

Desktop file organizer built with Rust, Tauri 2, and Svelte 5. Select an organization root, choose a grouping strategy, preview the plan, organize loose files into category folders, and undo when needed.

## Features

- **LooseRootScan** — organizes only loose, Finder-visible files sitting directly in the folder you select; subfolders, hidden files, symlinks, and existing category folders are skipped
- **Grouping strategies** — by semantic file type (Image, Video, Document, …), file size bucket, or modified date
- **Adaptive performance** — `RuntimeProfile` tiers (Low / Standard / High) tune walk threads, grouping parallelism, and move concurrency from CPU count and available RAM
- **Preview scan cache** — reuses the last preview for the same directory and strategy so organize does not rescan
- **Preview dialog and dry-run** — review the plan before moving anything; **Preview only** skips writes
- **Streaming undo logs** — batched JSON undo logs for large jobs, written into the organization root
- **Session undo** — restore the last operation from the current app session
- **Native desktop UI** — split-pane layout with directory picker, drag-and-drop, and dark styling (macOS, Linux, Windows)

## Tech stack

| Layer | Location | Role |
| --- | --- | --- |
| Core library | `backend/core/` | Walk, group, move, undo, naming, runtime tuning |
| Tauri shell | `backend/app/` | Commands, job orchestration, scan cache, undo history |
| Frontend | `frontend/` | Svelte 5 UI over Tauri IPC |
| Build | `build/` | Dev/build/test scripts, icons, CI templates |
| Tests | `test/fixtures/`, `backend/core/tests/` | Shared fixtures and Rust integration tests |
| Docs | `docs/` | Domain glossary (`CONTEXT.md`) and ADRs |

## Repository layout

```
frontend/              Svelte 5 + Vite UI
  src/lib/components/  Panels, controls, results
  src/lib/api/         Tauri command wrappers
backend/
  core/src/            Filesystem logic (walk, group, move, undo, file_type, runtime)
  app/src/             Tauri commands and organize jobs
  app/tauri.conf.json  App window and bundle config
build/
  scripts/             dev.sh, build.sh, test.sh
  icons/               Source icon assets
test/fixtures/         flat/, nested/ sample directories for core tests
docs/
  CONTEXT.md           Domain terminology
  adr/                 Architecture decision records
```

Category folders are created as `{CATEGORY}_FILES` (for example `PDF_FILES`, `IMAGE_FILES`) or `no_extension`, each marked with `.ocorganize-category` so later scans skip them.

## Prerequisites

- Rust stable toolchain
- Node.js 22+
- Platform dependencies for [Tauri 2](https://v2.tauri.app/start/prerequisites/)

### macOS

Xcode command line tools.

### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

### Windows

Microsoft Edge WebView2 runtime.

## Development

```bash
npm install
npm run dev
```

`npm run dev` installs frontend dependencies, starts the Vite dev server on port 5173, and launches Tauri dev mode with an unoptimized Rust debug build.

## Build

```bash
npm run build
```

Builds the frontend, then produces release bundles under `target/release/bundle/` (`.app`, `.dmg`, `.deb`, `.msi`, etc., depending on platform).

Use a release build when measuring performance; debug dev builds are significantly slower on large folders.

## Test

```bash
npm run test
```

Runs `cargo test --workspace` and frontend type checks (`svelte-check`).

Fixtures live under `test/fixtures/`; see [test/README.md](test/README.md).

## Usage

1. Select or drop a folder (the **organization root**).
2. Choose a strategy: **Type**, **Size**, or **Date**.
3. Click **Preview** to review the plan, or **Organize Files** to run.
4. Enable **Preview only** for a dry run that does not move files.
5. Leave **Create undo log** enabled to write a reversible log in the organization root.
6. Use **Undo** to restore the last operation from this app session. Undo log files remain on disk for manual recovery.

Domain terms are defined in [docs/CONTEXT.md](docs/CONTEXT.md).

## Environment

| Variable | Purpose |
| --- | --- |
| `OCORGANIZE_PERF_TIER` | Override auto-detected tier: `low`, `standard`, or `high` |

On macOS, `sysinfo` may report 0 bytes of available memory; tier detection falls back to free/total RAM. Set `OCORGANIZE_PERF_TIER` when benchmarking or if auto-detection picks the wrong tier.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)

## Troubleshooting

- **Build fails on Linux** — install the WebKitGTK packages listed above.
- **Drag and drop** — use the directory picker if the OS does not expose folder paths to the webview.
- **Large folders (100k+ files)** — performance scales with your machine. Preview once before organizing to reuse the cached scan. Set `OCORGANIZE_PERF_TIER=low|standard|high` to override auto-detection when testing.
- **Slow on a fast Mac** — tier detection falls back to total RAM when `sysinfo` reports 0 available memory (common on macOS). Use a release build (`npm run build`) for representative speed; `npm run dev` uses unoptimized Rust.
- **Permission errors** — the selected directory must be readable and writable.
