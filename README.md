# ArguMap Studio

[![TypeScript](https://img.shields.io/badge/typescript-%233178c6?style=flat-square&logo=typescript)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> Untangle complex arguments by making their structure visible — local, fast, and export-ready.

ArguMap Studio is a local-first argument mapping tool for macOS. Build structured argument graphs with typed nodes and edges, manage multiple maps, and export to PNG or JSON — all stored locally in SQLite with no network calls.

## Features

- **4 node types** — Claim, Evidence (with source attribution), Rebuttal, Counter-Rebuttal
- **4 edge types** — supports, rebuts, qualifies, depends_on — each color-coded
- **Interactive canvas** — drag to reposition, resize via handle, zoom and pan (powered by @xyflow/react)
- **Map library** — create, rename, delete, and switch between multiple argument maps
- **Sidebar editor** — edit node content, type, and source; change node type after creation
- **Export** — PNG (full canvas) and JSON (full serialized map)
- **Auto-save** — debounced writes to SQLite on every change; survives force-quit
- **Keyboard shortcuts** — `C`/`E`/`R`/`Shift+R` to add nodes, `Backspace` to delete, `Cmd+Z` to undo, `Cmd+E` to export PNG

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

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri 2 (Rust) |
| Frontend | React 18 + TypeScript 5.6 + Tailwind CSS 3 |
| Graph canvas | @xyflow/react 12 |
| PNG export | html-to-image |
| Persistence | SQLite via Tauri SQL plugin |

## License

MIT
