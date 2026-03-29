# ArguMap Studio

[![Version](https://img.shields.io/badge/version-0.0.0-blue)](package.json)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange)](https://tauri.app)
[![React](https://img.shields.io/badge/React-18-61DAFB?logo=react)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript)](https://www.typescriptlang.org)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey?logo=apple)](https://developer.apple.com/macos)

A local-first argument mapping tool for macOS. Build structured argument graphs with typed nodes and edges, manage multiple maps, and export to PNG or JSON — all stored locally in SQLite with no network calls.

---

![Screenshot placeholder](docs/screenshot.png)

> _Screenshot coming soon._

---

## Features

- **4 node types** — Claim, Evidence (with source attribution), Rebuttal, Counter-Rebuttal
- **4 edge types** — supports, rebuts, qualifies, depends_on — each color-coded
- **Interactive canvas** — drag to reposition, resize via handle, zoom and pan
- **Edge type modal** — select relationship type on every new connection
- **Map library** — create, rename, delete, and switch between multiple maps
- **Sidebar editor** — edit node content, type, and source from a panel; change node type after creation
- **Export** — PNG (full canvas, all nodes) and JSON (full serialized map)
- **Keyboard shortcuts** — `C` / `E` / `R` / `Shift+R` to add nodes, `Backspace` to delete, `Cmd+Z` to undo last action, `Cmd+E` to export PNG
- **Auto-save** — debounced writes to SQLite on every change; survives force-quit

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 (Rust) |
| Frontend | React 18 + TypeScript 5.6 |
| Canvas | React Flow (`@xyflow/react` v12) |
| Styling | Tailwind CSS v3 |
| Database | SQLite via `rusqlite` (bundled), stored at `~/.argumap/argumap.db` |
| Build | Vite 6 |
| Export | `html-to-image` for PNG |

## Prerequisites

- **macOS** (only supported platform — no Windows or Linux builds)
- [Node.js](https://nodejs.org) 18+
- [Rust](https://rustup.rs) (stable toolchain)
- [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/) — Xcode Command Line Tools

## Getting Started

```bash
# 1. Install frontend dependencies
npm install

# 2. Start development mode (opens native window with hot-reload)
npm run tauri dev

# 3. Build a release .dmg
npm run tauri build
```

The app creates `~/.argumap/argumap.db` on first launch and runs schema migrations automatically. No configuration required.

## Project Structure

```
argumap-studio/
├── src/
│   ├── components/
│   │   ├── canvas/          # ArgCanvas, custom node components, TypedEdge, EdgeTypeModal
│   │   └── sidebar/         # MapLibrary, AddNodePanel, NodeEditor
│   ├── hooks/
│   │   ├── useMapSync.ts    # Debounced React Flow → Tauri IPC writes
│   │   ├── useMapLoader.ts  # Load map from SQLite into React Flow state
│   │   └── useUndo.ts       # Single-level undo
│   ├── lib/
│   │   ├── tauri.ts         # Typed invoke() wrappers — all IPC goes here
│   │   ├── nodeFactory.ts   # Node creation helpers
│   │   └── exportUtils.ts   # PNG and JSON export logic
│   ├── types/index.ts       # Shared TypeScript interfaces and constants
│   └── App.tsx              # Root component; manages active map
├── src-tauri/
│   ├── src/
│   │   ├── commands.rs      # Tauri command implementations
│   │   ├── db.rs            # SQLite connection and migration runner
│   │   └── models.rs        # Rust structs mirroring TypeScript types
│   └── migrations/
│       └── 001_initial.sql  # Schema: maps, nodes, edges tables
└── package.json
```

## Data Storage

All data is stored locally at `~/.argumap/argumap.db`. No network calls are made — the Tauri `http` capability is intentionally not enabled. PNG and JSON exports are written to your local filesystem via a save dialog.

## License

MIT — see [LICENSE](LICENSE).
