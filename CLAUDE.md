# ArguMap Studio

Local-first macOS desktop app for interactive argument maps — Claims, Evidence, Rebuttals, and Counter-Rebuttals connected by typed edges. No cloud, no accounts. All data at `~/.argumap/argumap.db`.

## Stack

- **Desktop shell:** Tauri 2.0
- **Frontend:** React 18+ hooks only
- **Language:** TypeScript 5.x strict mode
- **Graph canvas:** @xyflow/react 12.x — React Flow (node/edge rendering, pan/zoom, NodeResizer)
- **Build tool:** Vite 6.x
- **Styling:** Tailwind CSS 3.x dark theme
- **DB driver (Rust):** rusqlite 0.31 (bundled feature)
- **Image export:** html-to-image 1.11
- **Database:** SQLite at `~/.argumap/argumap.db`

## Build / Run

```bash
# Development
npm run tauri dev

# Build release app
npm run tauri build
```

## Architecture Decisions

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

## Conventions

- TypeScript strict mode — type with `unknown` + narrowing; no `any`
- File naming: kebab-case files, PascalCase components
- All shared types belong in `src/types/index.ts`; define no types inline in component files
- All Tauri IPC goes through `src/lib/tauri.ts` wrappers; call no `invoke()` directly in components
- Conventional commits: `feat:`, `fix:`, `chore:`, `refactor:`
- Auto-save only — no manual save UI, no "unsaved changes" indicator

## Gotchas

- **One-way sync gate:** after initial hydration, write React Flow → SQLite only; writing back from SQLite into React Flow causes onChange loops
- **IPC gate:** all `invoke()` calls go through `src/lib/tauri.ts`; direct calls in components bypass the abstraction layer
- **Network capability disabled:** the Tauri `http` capability is intentionally absent — this app makes zero network calls; keep it that way
- **Canvas primitives:** React Flow provides all canvas primitives; build no graph canvas from scratch
- **Scope gate:** add only features in the current phase of `IMPLEMENTATION-ROADMAP.md`; out-of-phase additions bypass the roadmap contract

## Status

v1.0 + v2 shipped — release frozen. See `docs/PORTFOLIO-DISPOSITION.md`.

<!-- portfolio-context:start -->
# Portfolio Context

## What This Project Is

A local-first macOS desktop app for building interactive argument maps — structured graphs of Claims, Evidence, Rebuttals, and Counter-Rebuttals connected by typed relationship edges. Built for personal use: technical architecture decisions, IT incident root-cause chains, and structured research synthesis. All data stays on disk; no cloud, no accounts.

## Current State

**v1.0 + v2 shipped — Release Frozen**
All phases complete. See docs/PORTFOLIO-DISPOSITION.md for full disposition.

## Stack

- **Desktop shell:** Tauri 2.0
- **Frontend:** React 18+ (hooks only, no class components)
- **Language:** TypeScript 5.x (strict mode)
- **Graph canvas:** @xyflow/react 12.x — React Flow; handles node/edge rendering, pan/zoom, connection handles, NodeResizer
- **Build tool:** Vite 6.x
- **Styling:** Tailwind CSS 3.x (dark theme, utility classes)
- **DB driver (Rust):** rusqlite 0.31 (bundled feature)
- **Image export:** html-to-image 1.11
- **Database:** SQLite at `~/.argumap/argumap.db`

## How To Run

```bash
# Development
npm run tauri dev

# Build release app
npm run tauri build
```

## Known Risks

- Do not write back from SQLite into React Flow state after initial hydration — one-way sync only after load
- Do not call `invoke()` directly in component files — all IPC goes through `src/lib/tauri.ts`
- Do not define TypeScript interfaces outside `src/types/index.ts`
- Do not add features not in the current phase of IMPLEMENTATION-ROADMAP.md
- Do not enable the Tauri `http` capability — this app makes zero network calls
- Do not use class components — hooks only throughout
- Do not build the graph canvas from scratch — React Flow provides all canvas primitives

## Next Recommended Move

Use this context plus the README and supporting docs to resume the next active task, then promote the repo beyond minimum-viable by capturing a dedicated handoff, roadmap, or discovery artifact.

<!-- portfolio-context:end -->
