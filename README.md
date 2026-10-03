# ArguMap Studio

[![TypeScript](https://img.shields.io/badge/typescript-%233178c6?style=flat-square&logo=typescript)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> Untangle complex arguments by making their structure visible — local, fast, and export-ready.

ArguMap Studio is a local-first argument mapping tool for macOS. Build structured argument graphs with typed nodes and edges, manage multiple maps, and export to PNG, JSON, or HTML — all stored locally in SQLite with no network calls.

## Features

- **4 node types** — Claim, Evidence (with source attribution), Rebuttal, Counter-Rebuttal
- **4 edge types** — supports, rebuts, qualifies, depends_on — each color-coded
- **Interactive canvas** — drag to reposition, resize via handle, zoom and pan (powered by @xyflow/react)
- **Map library** — create, rename, delete, and switch between multiple argument maps
- **Sidebar editor** — edit node content, type, and source; change node type after creation
- **Export** — PNG (full canvas), JSON (full serialized map), and self-contained HTML
- **Strength scoring** — 1–5 rating per node, color-coded red→green, persisted to SQLite
- **Templates** — 5 Whys, Pro/Con, MECE starter graphs
- **Collapse/expand subtrees** — fold subtree branches to reduce clutter
- **Auto-save** — debounced writes to SQLite on every change; survives force-quit
- **Keyboard shortcuts** — `C`/`E`/`R`/`Shift+R` to add nodes, `Backspace` to delete, `Cmd+Z` to undo, `Cmd+E` to export PNG, `Cmd+Shift+E` to export HTML

## Quick Start

### Prerequisites
- Rust stable toolchain
- Node.js 20+ and npm

### Installation
```bash
git clone https://github.com/saagpatel/ArguMap
cd ArguMap
npm install
```

### Usage
```bash
# Development
npm run tauri dev

# Build release app
npm run tauri build
```

## Verification

Run from the repository root using Node.js 20+ and the checked-in npm lockfile:

```bash
npm ci
npm run build                       # TypeScript check and Vite bundle
cargo check --locked --manifest-path src-tauri/Cargo.toml
cargo test --locked --manifest-path src-tauri/Cargo.toml
```

Rust/Tauri checks need the stable Rust toolchain and macOS developer tools;
other platforms require their Tauri native build dependencies. `npm run build`
is the frontend typecheck/build gate. No JavaScript test runner, lint or format
script is configured; do not use `make test` or `make lint` as passing gates.
The Rust test command runs any Rust tests present (currently no unit tests),
so a zero-test result does not establish behavior coverage. No CI test workflow
is currently configured.

For UI changes, use `npm run dev` for browser checks of canvas/editor rendering.
For persistence, export or IPC changes, use `npm run tauri dev` in a disposable
macOS user profile with synthetic maps: the native app writes to its application
data directory. A browser-only run cannot verify SQLite/Tauri behavior. Check
the affected map/edit/export flow and never use personal maps as test fixtures.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri 2 (Rust) |
| Frontend | React 18 + TypeScript 5.6 + Tailwind CSS 3 |
| Graph canvas | @xyflow/react 12 |
| PNG export | html-to-image |
| Persistence | SQLite via rusqlite (bundled) |

## License

MIT
