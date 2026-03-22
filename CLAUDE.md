# ArguMap Studio

## Overview
A local-first macOS desktop app for building interactive argument maps — structured graphs of Claims, Evidence, Rebuttals, and Counter-Rebuttals connected by typed relationship edges. Built for personal use: technical architecture decisions, IT incident root-cause chains, and structured research synthesis. All data stays on disk; no cloud, no accounts.

## Tech Stack
- **Desktop shell:** Tauri 2.0
- **Frontend:** React 18+ (hooks only, no class components)
- **Language:** TypeScript 5.x (strict mode)
- **Graph canvas:** @xyflow/react 12.x — React Flow; handles node/edge rendering, pan/zoom, connection handles, NodeResizer
- **Build tool:** Vite 5.x
- **Styling:** Tailwind CSS 3.x (dark theme, utility classes)
- **DB driver (Rust):** rusqlite 0.31 (bundled feature)
- **Image export:** html-to-image 1.11
- **Database:** SQLite at `~/.argumap/argumap.db`

## Development Conventions
- TypeScript strict mode — no `any` types, no implicit returns
- File naming: kebab-case for files, PascalCase for React components
- All shared types in `src/types/index.ts` — never define types inline in component files
- All Tauri `invoke()` calls go through `src/lib/tauri.ts` wrappers — never call `invoke()` directly in components
- Conventional commits: `feat:`, `fix:`, `chore:`, `refactor:`
- Auto-save only — no manual save UI, no "unsaved changes" indicator

## Current Phase
**Phase 0: Scaffold + Database (Days 1–2)**
See IMPLEMENTATION-ROADMAP.md for full phase details and acceptance criteria.

## Key Decisions
| Decision | Choice | Why |
|---|---|---|
| State management | Hybrid: React Flow owns visual state; SQLite owns content/structure | Avoids re-render loops; React Flow is authoritative for rendering, SQLite is authoritative for persistence |
| Sync direction | One-way after hydration: React Flow → SQLite only | Writing SQLite back into React Flow after initial load causes onChange loops |
| Auto-save cadence | Position/resize: 500ms debounce; Content: 1000ms debounce; Edges: immediate | Position changes are frequent; content is deliberate; edges are structural |
| IPC save pattern | Full state upsert: all nodes + edges for a map in one transaction | Simpler than delta sync; SQLite upsert handles the scale |
| Counter-rebuttal | Distinct node type with its own Add button | Semantically different from Rebuttal (attacks a rebuttal, not a claim); distinct orange visual |
| Node resize | In scope v1 via NodeResizer; min 160×60px | Width/height persisted in `nodes` table as `width REAL` / `height REAL` |
| Undo scope | Single last action only | Full history is v2; single-level covers 90% of "oops" cases |
| Duplicate edges | Prevented silently | Same source + target + edge_type = no-op |
| Launch behavior | Load most recent map by `updated_at DESC` | Zero friction; fresh install creates an untitled map |

## Do NOT
- Do not write back from SQLite into React Flow state after initial hydration — one-way sync only after load
- Do not call `invoke()` directly in component files — all IPC goes through `src/lib/tauri.ts`
- Do not define TypeScript interfaces outside `src/types/index.ts`
- Do not add features not in the current phase of IMPLEMENTATION-ROADMAP.md
- Do not enable the Tauri `http` capability — this app makes zero network calls
- Do not use class components — hooks only throughout
- Do not build the graph canvas from scratch — React Flow provides all canvas primitives
