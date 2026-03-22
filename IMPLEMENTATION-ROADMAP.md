# ArguMap Studio — Implementation Roadmap

---

## Architecture

### System Overview

```
[React Frontend]
  └── ArgCanvas.tsx (React Flow root)
        ├── Custom Nodes: ClaimNode / EvidenceNode / RebuttalNode / CounterRebuttalNode
        │     └── NodeResizer (each node)
        ├── Custom Edges: TypedEdge (color + label by edge_type)
        ├── EdgeTypeModal (opens on connection drop)
        └── onNodesChange / onEdgesChange
              └── useMapSync.ts (debounced) → invoke('save_map_state') → SQLite

[React Frontend]
  └── Sidebar.tsx
        ├── MapLibrary.tsx (map list, new map, rename, delete)
        ├── AddNodePanel.tsx ([+Claim] [+Evidence] [+Rebuttal] [+Counter-Rebuttal])
        └── NodeEditor.tsx (shown when node selected: content, type, source)

[Tauri Backend (Rust)]
  ├── commands.rs → get_maps / create_map / delete_map / rename_map / load_map / save_map_state / export_map_json
  └── db.rs → rusqlite connection + 001_initial.sql migration

[File System]
  └── ~/.argumap/argumap.db (SQLite)
```

### File Structure

```
argumap-studio/
├── src/
│   ├── components/
│   │   ├── canvas/
│   │   │   ├── ArgCanvas.tsx          # React Flow root; owns useNodesState/useEdgesState
│   │   │   ├── nodes/
│   │   │   │   ├── ClaimNode.tsx
│   │   │   │   ├── EvidenceNode.tsx   # adds source attribution input
│   │   │   │   ├── RebuttalNode.tsx
│   │   │   │   └── CounterRebuttalNode.tsx
│   │   │   ├── edges/
│   │   │   │   └── TypedEdge.tsx      # color + label from EDGE_COLORS[edge_type]
│   │   │   └── EdgeTypeModal.tsx      # modal on new connection; selects edge type
│   │   └── sidebar/
│   │       ├── Sidebar.tsx            # container; switches between panels
│   │       ├── MapLibrary.tsx         # map list, new map, right-click rename/delete
│   │       ├── AddNodePanel.tsx       # [+Claim] [+Evidence] [+Rebuttal] [+Counter-Rebuttal]
│   │       └── NodeEditor.tsx         # selected node: content textarea, type select, source input
│   ├── hooks/
│   │   ├── useMapSync.ts              # debounced React Flow → Tauri IPC writes
│   │   ├── useMapLoader.ts            # SQLite → hydrate React Flow (one-time per mapId)
│   │   └── useUndo.ts                 # single lastAction ref; Cmd+Z reverts
│   ├── lib/
│   │   ├── tauri.ts                   # typed invoke() wrappers — ALL IPC goes here
│   │   ├── nodeFactory.ts             # createNode(type, viewportCenter) → ArgNode
│   │   └── exportUtils.ts             # PNG (fitView + html-to-image) + JSON export
│   ├── types/
│   │   └── index.ts                   # ALL shared TypeScript interfaces and constants
│   ├── App.tsx                        # root; manages activeMapId; startup map load
│   └── main.tsx                       # Vite entry point
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                    # Tauri app setup; registers all commands
│   │   ├── commands.rs                # all #[tauri::command] implementations
│   │   ├── db.rs                      # init_db(), connection pool, migration runner
│   │   └── models.rs                  # Rust structs: Map, ArgNode, ArgEdge, NodePayload, EdgePayload
│   ├── migrations/
│   │   └── 001_initial.sql
│   └── tauri.conf.json                # bundle ID: com.argumap.studio; NO http capability
├── CLAUDE.md
├── IMPLEMENTATION-ROADMAP.md
├── package.json
└── tsconfig.json
```

### Data Model

```sql
-- src-tauri/migrations/001_initial.sql

CREATE TABLE maps (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL DEFAULT 'Untitled Map',
  description TEXT,
  created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_maps_updated ON maps(updated_at DESC);

CREATE TABLE nodes (
  id          TEXT PRIMARY KEY,
  map_id      TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
  node_type   TEXT NOT NULL CHECK(node_type IN ('claim','evidence','rebuttal','counter_rebuttal')),
  content     TEXT NOT NULL DEFAULT '',
  source      TEXT,          -- citation; meaningful for evidence nodes only
  x           REAL NOT NULL DEFAULT 100,
  y           REAL NOT NULL DEFAULT 100,
  width       REAL NOT NULL DEFAULT 220,
  height      REAL NOT NULL DEFAULT 80,
  created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_nodes_map ON nodes(map_id);

CREATE TABLE edges (
  id             TEXT PRIMARY KEY,
  map_id         TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
  source_node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  target_node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
  edge_type      TEXT NOT NULL CHECK(edge_type IN ('supports','rebuts','qualifies','depends_on')),
  label          TEXT,
  UNIQUE(source_node_id, target_node_id, edge_type)   -- prevents duplicate edges
);
CREATE INDEX idx_edges_map ON edges(map_id);
```

### TypeScript Type Definitions

```typescript
// src/types/index.ts — canonical source of truth for all types and constants

export type NodeType = 'claim' | 'evidence' | 'rebuttal' | 'counter_rebuttal';
export type EdgeType = 'supports' | 'rebuts' | 'qualifies' | 'depends_on';

export interface ArgMap {
  id: string;
  title: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface ArgNode {
  id: string;
  map_id: string;
  node_type: NodeType;
  content: string;
  source?: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ArgEdge {
  id: string;
  map_id: string;
  source_node_id: string;
  target_node_id: string;
  edge_type: EdgeType;
  label?: string;
}

// IPC payload sent to save_map_state — matches Rust NodePayload struct exactly
export interface NodePayload {
  id: string;
  node_type: NodeType;
  content: string;
  source?: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

// IPC payload sent to save_map_state — matches Rust EdgePayload struct exactly
export interface EdgePayload {
  id: string;
  source_node_id: string;
  target_node_id: string;
  edge_type: EdgeType;
  label?: string;
}

// Stored in React Flow node.data field
export interface ArgNodeData {
  node_type: NodeType;
  content: string;
  source?: string;
  onUpdate: (id: string, updates: Partial<Omit<ArgNodeData, 'onUpdate'>>) => void;
}

// Centralized color constants — reference these everywhere, never hardcode hex in components
export const EDGE_COLORS: Record<EdgeType, string> = {
  supports:   '#10B981',
  rebuts:     '#EF4444',
  qualifies:  '#FBBF24',
  depends_on: '#6B7280',
};

export const NODE_CONFIG: Record<NodeType, { border: string; bg: string; label: string }> = {
  claim:            { border: '#3B82F6', bg: '#1E3A5F', label: 'Claim' },
  evidence:         { border: '#10B981', bg: '#0F3028', label: 'Evidence' },
  rebuttal:         { border: '#EF4444', bg: '#3B1212', label: 'Rebuttal' },
  counter_rebuttal: { border: '#F97316', bg: '#3B1E0A', label: 'Counter-Rebuttal' },
};

// Default node dimensions
export const NODE_DEFAULTS = {
  width: 220,
  height: 80,
  minWidth: 160,
  minHeight: 60,
} as const;
```

### Rust Struct Definitions

```rust
// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Map {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArgNode {
    pub id: String,
    pub map_id: String,
    pub node_type: String,
    pub content: String,
    pub source: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArgEdge {
    pub id: String,
    pub map_id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: String,
    pub label: Option<String>,
}

// Incoming IPC payload from frontend — must match TypeScript NodePayload exactly
#[derive(Debug, Deserialize)]
pub struct NodePayload {
    pub id: String,
    pub node_type: String,
    pub content: String,
    pub source: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

// Incoming IPC payload from frontend — must match TypeScript EdgePayload exactly
#[derive(Debug, Deserialize)]
pub struct EdgePayload {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: String,
    pub label: Option<String>,
}
```

### Tauri IPC Contracts

```typescript
// src/lib/tauri.ts — ALL invoke() calls go here, never in component files

import { invoke } from '@tauri-apps/api/core';
import type { ArgMap, ArgNode, ArgEdge, NodePayload, EdgePayload } from '../types';

export const tauriApi = {
  getMaps: () =>
    invoke<ArgMap[]>('get_maps'),

  createMap: (title: string, description?: string) =>
    invoke<ArgMap>('create_map', { title, description }),

  deleteMap: (mapId: string) =>
    invoke<void>('delete_map', { mapId }),

  renameMap: (mapId: string, title: string) =>
    invoke<void>('rename_map', { mapId, title }),

  loadMap: (mapId: string) =>
    invoke<{ nodes: ArgNode[]; edges: ArgEdge[] }>('load_map', { mapId }),

  saveMapState: (mapId: string, nodes: NodePayload[], edges: EdgePayload[]) =>
    invoke<void>('save_map_state', { mapId, nodes, edges }),

  exportMapJson: (mapId: string) =>
    invoke<string>('export_map_json', { mapId }),
};
```

### Tauri Command Signatures (commands.rs)

```rust
// All 7 commands must be registered in main.rs via .invoke_handler(tauri::generate_handler![...])

#[tauri::command]
async fn get_maps(db: State<'_, DbPool>) -> Result<Vec<Map>, String>
// Returns all maps sorted by updated_at DESC

#[tauri::command]
async fn create_map(db: State<'_, DbPool>, title: String, description: Option<String>) -> Result<Map, String>
// Generates UUID v4 id; inserts into maps; returns full Map struct

#[tauri::command]
async fn delete_map(db: State<'_, DbPool>, map_id: String) -> Result<(), String>
// Cascading delete handled by SQLite FK ON DELETE CASCADE

#[tauri::command]
async fn rename_map(db: State<'_, DbPool>, map_id: String, title: String) -> Result<(), String>
// Updates title and updated_at

#[tauri::command]
async fn load_map(db: State<'_, DbPool>, map_id: String) -> Result<serde_json::Value, String>
// Returns { nodes: ArgNode[], edges: ArgEdge[] }

#[tauri::command]
async fn save_map_state(db: State<'_, DbPool>, map_id: String, nodes: Vec<NodePayload>, edges: Vec<EdgePayload>) -> Result<(), String>
// Full upsert: DELETE all nodes+edges for map_id, INSERT all from payload, in one transaction
// Also updates maps.updated_at

#[tauri::command]
async fn export_map_json(db: State<'_, DbPool>, map_id: String) -> Result<String, String>
// Returns serde_json::to_string of full { map, nodes, edges } struct
```

### Key Hook: useMapSync.ts

```typescript
// src/hooks/useMapSync.ts
// Critical: isHydrating guard prevents write loops during initial map load

import { useRef, useCallback } from 'react';
import { useReactFlow } from '@xyflow/react';
import { tauriApi } from '../lib/tauri';
import type { NodePayload, EdgePayload } from '../types';

export function useMapSync(mapId: string | null) {
  const isHydrating = useRef(false);       // true during setNodes/setEdges hydration
  const positionDebounceRef = useRef<ReturnType<typeof setTimeout>>();
  const contentDebounceRef = useRef<ReturnType<typeof setTimeout>>();
  const { getNodes, getEdges } = useReactFlow();

  const flush = useCallback(() => {
    if (!mapId || isHydrating.current) return;
    const nodes: NodePayload[] = getNodes().map(n => ({
      id: n.id,
      node_type: n.data.node_type,
      content: n.data.content,
      source: n.data.source,
      x: n.position.x,
      y: n.position.y,
      width: n.measured?.width ?? n.data.width ?? 220,
      height: n.measured?.height ?? n.data.height ?? 80,
    }));
    const edges: EdgePayload[] = getEdges().map(e => ({
      id: e.id,
      source_node_id: e.source,
      target_node_id: e.target,
      edge_type: e.data.edge_type,
      label: e.data.label,
    }));
    tauriApi.saveMapState(mapId, nodes, edges).catch(console.error);
  }, [mapId, getNodes, getEdges]);

  const syncOnPosition = useCallback(() => {
    clearTimeout(positionDebounceRef.current);
    positionDebounceRef.current = setTimeout(flush, 500);
  }, [flush]);

  const syncOnContent = useCallback(() => {
    clearTimeout(contentDebounceRef.current);
    contentDebounceRef.current = setTimeout(flush, 1000);
  }, [flush]);

  const syncImmediate = useCallback(() => {
    flush();
  }, [flush]);

  return { isHydrating, syncOnPosition, syncOnContent, syncImmediate };
}
```

### Dependencies

```bash
# 1. Scaffold
npm create tauri-app@latest argumap-studio -- --template react-ts
cd argumap-studio

# 2. Frontend dependencies
npm install @xyflow/react@12 html-to-image uuid
npm install -D @types/uuid

# 3. Tailwind CSS
npm install -D tailwindcss@3 postcss autoprefixer
npx tailwindcss init -p
# In tailwind.config.js: darkMode: 'class', content: ['./index.html', './src/**/*.{ts,tsx}']
# In index.html: add class="dark" to <html>

# 4. Rust dependencies — add to src-tauri/Cargo.toml under [dependencies]
# rusqlite = { version = "0.31", features = ["bundled"] }
# uuid = { version = "1", features = ["v4"] }
# chrono = { version = "0.4", features = ["serde"] }
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
```

---

## Scope Boundaries

**In scope (v1):**
- All 4 node types: Claim, Evidence, Rebuttal, Counter-Rebuttal
- All 4 edge types: supports, rebuts, qualifies, depends_on
- Node drag-to-reposition, resize via NodeResizer
- Edge creation via connection handle drag → EdgeTypeModal
- Map library: create, rename, delete, switch between maps
- Inline map title editing in menu bar
- Selected node editor in sidebar (content, type change, source)
- Export to PNG (all nodes) and JSON
- Keyboard shortcuts: C / E / R / Shift+R / Backspace / Delete / Cmd+Z / Cmd+E
- Single-level undo (last add/delete action)
- Auto-save to SQLite on every change (debounced)
- Empty state onboarding prompt on new maps
- macOS .dmg packaging

**Out of scope (v1, do not build):**
- Cloud sync or sharing
- Import from JSON
- Collaborative editing
- Windows or Linux builds
- Any network calls

**Deferred to v2:**
- Templates (5 Whys, Pro/Con, MECE)
- Export as self-contained shareable HTML
- Strength scoring (1–5) on nodes
- Full undo history (multi-step)
- Collapse/expand subtrees
- Map search across all maps
- SQLCipher encryption at rest

---

## Security & Credentials

- **No credentials.** Zero network calls. Tauri `http` capability NOT enabled in `tauri.conf.json`.
- **Database:** `~/.argumap/argumap.db` — user home directory, not world-readable by default
- **Data at rest:** Unencrypted SQLite in v1. v2 option: SQLCipher if sensitive content warrants it.
- **Data leaving machine:** None. PNG/JSON exports write to user's local filesystem only via save dialog.

---

## Phase 0: Scaffold + Database (Days 1–2)

**Objective:** Working Tauri 2 + React + TypeScript + Tailwind shell. SQLite initialized at `~/.argumap/argumap.db` with full schema. All 7 Tauri commands stubbed and callable.

**Tasks:**
1. Run `npm create tauri-app` with `react-ts` template — **Acceptance:** `npm run tauri dev` opens Tauri window with no console errors
2. Install all dependencies per dependency list above — **Acceptance:** `@xyflow/react` importable in App.tsx; `npm run build` completes
3. Configure Tailwind: `darkMode: 'class'`; add `dark` class to `<html>` in index.html — **Acceptance:** Test div with `className="bg-zinc-900 text-white"` renders correctly
4. Create `db.rs` with `init_db()` that creates `~/.argumap/` directory and opens/creates `argumap.db` — **Acceptance:** `ls ~/.argumap/` shows `argumap.db` after first launch
5. Write and run `001_initial.sql` migration on startup — **Acceptance:** `sqlite3 ~/.argumap/argumap.db ".schema"` shows all 3 tables with correct columns and UNIQUE constraint on edges
6. Implement all 7 Tauri commands in `commands.rs`; register in `main.rs` — **Acceptance:** `invoke('get_maps')` from browser devtools returns `[]`; `invoke('create_map', { title: 'Test' })` returns map object with UUID id
7. Write `src/types/index.ts` (all interfaces + EDGE_COLORS + NODE_CONFIG + NODE_DEFAULTS) and `src/lib/tauri.ts` (all 7 wrappers) — **Acceptance:** `npm run build` = 0 TypeScript errors

**Verification checklist:**
- [ ] `ls ~/.argumap/argumap.db` — file exists
- [ ] `sqlite3 ~/.argumap/argumap.db ".tables"` → `edges  maps  nodes`
- [ ] `sqlite3 ~/.argumap/argumap.db ".schema edges"` → UNIQUE constraint visible
- [ ] `npm run build` → 0 TypeScript errors
- [ ] Browser devtools: `invoke('create_map', { title: 'Test' })` → returns `{ id: '<uuid>', title: 'Test', ... }`

**Risks:**
- rusqlite `bundled` feature adds compile time — expected; first `cargo build` takes 2–3 min on M4 Pro
- Tauri 2 `invoke()` parameter casing: Rust uses snake_case, TypeScript invoke args use camelCase → Tauri handles conversion automatically via serde; confirm with a test call

---

## Phase 1: Canvas + Node Foundation (Weeks 1–2)

**Objective:** Interactive React Flow canvas with all 4 custom node types. Nodes can be added, repositioned, resized, edited, and deleted. Full hybrid sync to SQLite. App loads most recent map on launch.

**Tasks:**
1. Build `ArgCanvas.tsx`: `useNodesState` / `useEdgesState`; dark canvas background `#0F0F0F`; `<MiniMap>` bottom-right; `<Controls>` visible — **Acceptance:** Canvas renders; pan, zoom, minimap, and controls all work
2. Build all 4 custom node components using `NODE_CONFIG` color spec: rounded rectangle with colored border + dark bg; type badge top-left; content `<textarea>` (auto-focus on new node); custom connection handles (4 directions, 24px hit target, glow ring on hover); `EvidenceNode` adds source attribution `<input>` below textarea — **Acceptance:** All 4 node types render with distinct colors; handles glow on node hover
3. Add `NodeResizer` from `@xyflow/react` to all 4 node components; enforce `minWidth={160}` `minHeight={60}`; expose `width`/`height` in `node.data` for sync — **Acceptance:** Resize handle visible on selected node; dimensions update during drag; min dimensions enforced
4. Build `AddNodePanel.tsx`: 4 buttons; click creates node via `nodeFactory.createNode(type, viewportCenter)` using `reactFlowInstance.screenToFlowPosition(viewportCenter)`; immediately enters edit mode — **Acceptance:** Clicking each button places correct node type at canvas center; textarea auto-focused
5. Implement `useMapSync.ts` (see Architecture section for full implementation); add `isHydrating` ref guard; debounced 500ms on position/resize, 1000ms on content — **Acceptance:** Edit content → wait 1.5s → force-quit → relaunch → content persisted. Confirm 0 writes during initial load via `console.log` in `flush()`
6. Build `useMapLoader.ts`: on `mapId` change, set `isHydrating.current = true` → call `tauriApi.loadMap(mapId)` → convert `ArgNode[]` to React Flow format → `setNodes()` → `setEdges()` → use `useEffect` cleanup or `setTimeout(0)` to set `isHydrating.current = false` — **Acceptance:** Switching maps renders correct nodes; no spurious DB writes during load (verify in console)
7. Build `App.tsx` startup: `tauriApi.getMaps()` → sort by `updated_at DESC` → load first map's ID; if empty, `tauriApi.createMap('Untitled Map')` then load — **Acceptance:** App opens last-used map on relaunch; fresh install shows empty canvas with map title "Untitled Map"
8. Node deletion: React Flow `onNodesDelete` handler; if node has connected edges, show confirmation: `"Delete this node? This will also remove N connected edge(s)."` (count from current edges array); confirmed → remove from state → `syncImmediate()` — **Acceptance:** Deleting node with 2 edges shows correct count; deletion removes node + edges from canvas and DB

**Verification checklist:**
- [ ] Add one of each node type → force-quit → relaunch → all 4 present with correct colors
- [ ] Resize a Claim node → force-quit → relaunch → node retains custom dimensions
- [ ] Edit Evidence node content + source → wait 2s → relaunch → both fields persisted
- [ ] Create 2 maps → switch between → each shows its own nodes
- [ ] Delete a node with edges → confirmation shows correct edge count

**Risks:**
- `onNodesChange` fires during `setNodes()` hydration → `isHydrating` ref guard in `useMapSync` suppresses writes during load. If guard misfires: fallback is a `hasArmedSync` boolean set to `true` only after first `pointerdown` event on the canvas
- React Flow handles invisible on dark canvas → custom handle components with explicit border + glow solve this; test hover state before moving to Phase 2

---

## Phase 2: Edges + Map Library (Weeks 3–4)

**Objective:** Full typed edge creation via handle drag. Edge type selection modal. Map library CRUD. Keyboard shortcuts.

**Tasks:**
1. Build `TypedEdge.tsx` custom edge component: renders `<BaseEdge>` with `stroke` from `EDGE_COLORS[edge_type]`; `strokeWidth: 2`; edge type label in `<EdgeLabelRenderer>` shown only on hover (CSS opacity transition); animated dashes on hover — **Acceptance:** All 4 edge types show correct colors; label appears/disappears on hover
2. Build `EdgeTypeModal.tsx`: triggered by React Flow `onConnect` callback; render as React portal on `document.body`; fixed position `top: 50%, left: 50%, transform: translate(-50%, -50%)`; 4 buttons with color swatches; Escape cancels; button click calls `addEdge()` + `syncImmediate()` — **Acceptance:** Dragging handle to target opens modal; selecting "rebuts" creates red edge; Escape creates no edge
3. Duplicate edge guard in `onConnect`: before showing modal, check `edges.some(e => e.source === params.source && e.target === params.target && e.data.edge_type === selectedType)`; block if exists — **Acceptance:** Creating second identical edge produces no duplicate in canvas or DB
4. Self-loop guard: pass `isValidConnection={(conn) => conn.source !== conn.target}` to `<ReactFlow>` — **Acceptance:** Dragging handle back to same node shows no connection formed
5. Build `MapLibrary.tsx`: `tauriApi.getMaps()` on mount + after any map mutation; sorted by `updated_at DESC`; active map highlighted with left border accent; `[+ New Map]` button; right-click context menu with Rename (inline input) and Delete (confirm dialog) — **Acceptance:** Create 3 maps, rename one, delete one; library reflects correct state; DB confirms via sqlite3
6. Inline map title rename in menu bar: click title → `<input>` replaces text with current title; Enter or blur → `tauriApi.renameMap()` — **Acceptance:** Rename persists after app restart
7. Keyboard shortcuts via `useEffect` on `document.addEventListener('keydown')`; suppress when `event.target` is `INPUT` or `TEXTAREA`: `C` → new Claim, `E` → new Evidence, `R` → new Rebuttal, `Shift+R` → new Counter-Rebuttal, `Backspace`/`Delete` → delete selected node/edge, `Cmd+Z` → undo, `Cmd+E` → export PNG — **Acceptance:** All shortcuts fire on canvas; none fire when typing in node textarea or sidebar inputs
8. Edge deletion: `onEdgesDelete` handler removes edge from state + `syncImmediate()` — no confirmation needed — **Acceptance:** Select edge → Backspace → removed from canvas and DB

**Verification checklist:**
- [ ] Connect Claim → Evidence → select "supports" → green edge with label on hover
- [ ] Create duplicate edge attempt → silently no-ops
- [ ] Drag handle to same source node → no self-loop formed
- [ ] Create 3 maps, rename one, delete one → sqlite3 confirms correct 2 rows in maps table
- [ ] `Shift+R` → Counter-Rebuttal node at canvas center, orange border
- [ ] `C` while typing in node textarea → no new node created

**Risks:**
- `EdgeTypeModal` renders before edge type is selected — store `pendingConnection` in state from `onConnect`; render modal when `pendingConnection !== null`; clear on confirm or cancel
- Map library re-fetch on every switch adds latency — cache `maps[]` in `App.tsx` state; refresh only on create/rename/delete mutations

---

## Phase 3: Export + Sidebar Editor + Polish (Weeks 5–6)

**Objective:** PNG and JSON export. Selected node editor in sidebar. Single-level undo. Empty state. Dark theme polish. macOS .dmg packaging.

**Tasks:**
1. Build `NodeEditor.tsx`: renders in sidebar when a node is selected (track via React Flow `onSelectionChange`); content `<textarea>` (auto-resize via `scrollHeight`); `<select>` for node_type (changing type calls `onUpdate` which updates `node.data.node_type` → React Flow re-renders with new node component); source `<input>` visible only when `node_type === 'evidence'` — **Acceptance:** Edit content in sidebar → canvas node text updates in real time; type change → new border color; source field appears/disappears by type
2. Node type switching: `onUpdate` callback updates React Flow node's `type` field (to the new node component name) and `data.node_type`; triggers `syncOnContent()` — **Acceptance:** Change Claim to Counter-Rebuttal → orange border on canvas; persists after restart
3. PNG export in `exportUtils.ts`: `reactFlowInstance.fitView({ padding: 0.1, duration: 0 })` → `await new Promise(resolve => setTimeout(resolve, 300))` → `toPng(document.querySelector('.react-flow__viewport'), { backgroundColor: '#0A0A0A', pixelRatio: 2 })` → trigger download as `${mapTitle}.png` — **Acceptance:** 10-node map exported PNG contains all 10 nodes; no nodes clipped at edges
4. JSON export: `tauriApi.exportMapJson(mapId)` → trigger download as `${mapTitle}.json`; file should open cleanly in VS Code — **Acceptance:** Exported JSON node count matches canvas node count
5. `useUndo.ts`: `lastAction` ref typed as `{ type: 'add_node' | 'delete_node' | 'add_edge' | 'delete_edge'; payload: ... } | null`; on `Cmd+Z`, revert the action (re-add deleted or delete newly added); set `lastAction = null` after undo; `syncImmediate()` after revert — **Acceptance:** Add node → Cmd+Z → node removed; delete node → Cmd+Z → node restored with correct content and position
6. Empty state: when `nodes.length === 0`, render centered overlay on canvas: large faded `🗺️` icon + `"Start by adding a Claim"` + `"Press C or click + Claim in the sidebar"` — overlay hides when first node is added — **Acceptance:** New map shows prompt; adding first node removes overlay
7. Dark theme final pass — apply to all surfaces: canvas `bg-[#0A0A0A]`; sidebar `bg-[#111111]`; sidebar border `border-[#222222]`; menu bar `bg-[#111111]`; all body text `text-[#E5E7EB]`; input backgrounds `bg-[#1C1C1C]`; scrollbars styled via `::webkit-scrollbar` — **Acceptance:** Full visual review — no light backgrounds visible; no unstyled native inputs
8. macOS packaging: configure `tauri.conf.json` with `productName: "ArguMap Studio"`, `identifier: "com.argumap.studio"`, add app icon (1024×1024 PNG); run `npm run tauri build` — **Acceptance:** `.dmg` produced in `src-tauri/target/release/bundle/dmg/`; app opens without Gatekeeper errors on M4 Pro

**Verification checklist:**
- [ ] Select Evidence node → source field appears in sidebar → edit → persists after restart
- [ ] Change Claim to Counter-Rebuttal via sidebar dropdown → orange border immediately
- [ ] Export PNG of 15-node map → all 15 nodes visible in image (verify pixel count / count nodes in image viewer)
- [ ] Export JSON → open in text editor → node count matches canvas
- [ ] Cmd+Z after node add → node removed; Cmd+Z after node delete → node restored with correct content
- [ ] Empty state visible on new map; disappears on first node add
- [ ] `npm run tauri build` → `.dmg` file exists and opens cleanly

**Risks:**
- `html-to-image` captures only visible viewport at current zoom → solved by `fitView()` before capture; if nodes still clip, increase `padding` to 0.2
- Node type switching: React Flow uses `type` field on node to pick component; ensure all 4 component names match keys in `nodeTypes` prop on `<ReactFlow>`

---

## Definition of Done (v1)

- [ ] Create a new argument map with a custom title
- [ ] Add all 4 node types: Claim, Evidence, Rebuttal, Counter-Rebuttal
- [ ] Connect nodes with all 4 edge types: supports, rebuts, qualifies, depends_on
- [ ] Reposition nodes freely; resize via drag handle (min 160×60px)
- [ ] Edit node content and source field inline on canvas and via sidebar editor
- [ ] Change a node's type after creation
- [ ] Delete nodes (with confirmation if edges exist) and edges (no confirmation)
- [ ] Manage multiple maps: create, rename, delete, switch
- [ ] All data persists across force-quit and relaunch with zero data loss
- [ ] Keyboard shortcuts: C / E / R / Shift+R / Backspace / Delete / Cmd+Z / Cmd+E
- [ ] Export to PNG (all nodes, not just viewport)
- [ ] Export to JSON (full serialized map)
- [ ] macOS .dmg builds and runs without errors on M4 Pro
