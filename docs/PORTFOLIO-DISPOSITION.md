# ArguMap (ArguMap Studio) — Portfolio Disposition

**Status:** Release Frozen — Tauri 2 + Rust + React (`@xyflow/react`)
desktop argument mapping app on `origin/main`. v1.0 + v2 shipped:
4 node types (Claim / Evidence / Rebuttal / Counter-Rebuttal),
4 edge types, drag canvas, map library, PNG / JSON export, auto-save
to SQLite, collapse/expand subtrees, templates, HTML export,
strength scoring (1-5). Joins the signing cluster as the 21st member.

> Disposition uses strict `origin/main` verification.

---

## Verification posture

This repo has **only `origin`** (`saagpatel/ArguMap`) — no
`legacy-origin` remote. Clean migration state. Local clone's `main`
is tracking `origin/main` correctly.

Specifically verified on `origin/main`:

- Tip: `72f4db3` (HEAD)
- Substantive commits on `origin/main`:
  - `c1bd868` feat: strength scoring (1-5) on nodes
  - `911a752` feat: v2 — collapse/expand subtrees, templates, HTML export
  - `f6f2e29` feat: ArguMap Studio v1.0 — complete argument mapping desktop app
  - `cacb483` fix: remove duplicate TemplateKey type, single source in templates.ts
  - `00a8de2` merge: v2 features into main
- Tree on `origin/main` is a full Tauri 2 desktop app:
  - `src-tauri/` (Rust backend)
  - `src/` (React + xyflow frontend)
  - `index.html`, `package.json`, `pnpm-lock.yaml` (project root)
  - `IMPLEMENTATION-ROADMAP.md`
- **Cross-platform packaging signals:** macOS `Square*Logo.png` +
  Android `mipmap-*/ic_launcher*.png` variants — Tauri mobile
  templates are present, hinting at Android distribution ambition
- Release scaffolding: none yet (no `RELEASE-READINESS.md`, no
  `release-smoke.yml`)
- Default branch: `main`

---

## Current state in one paragraph

ArguMap Studio is a Tauri 2 + Rust + React desktop app for argument
mapping. Per README: 4 node types (Claim, Evidence with source
attribution, Rebuttal, Counter-Rebuttal), 4 edge types (supports,
rebuts, qualifies, depends_on, each color-coded), interactive
canvas via `@xyflow/react` with drag / resize / zoom / pan, map
library (create / rename / delete / switch), sidebar editor,
PNG + JSON export, debounced auto-save to SQLite. v2 added
collapse/expand subtrees, templates, HTML export, and 1-5 strength
scoring on nodes. Keyboard shortcuts cover `C`/`E`/`R`/`Shift+R`
add, `Backspace` delete, `Cmd+Z` undo, `Cmd+E` PNG export. Memory's
"v1+v2 complete" claim matches canonical state.

For full detail see:
- `README.md` on `origin/main`
- `IMPLEMENTATION-ROADMAP.md`

---

## Why "Release Frozen" instead of other dispositions

- **Active** — wrong. v1 + v2 are both shipped; the gate is signing,
  not feature delivery.
- **Cold Storage / Archived** — wrong. v2 commits are recent product
  work.
- **Release Frozen** — correct. Joins the cluster.

This is the **21st signing cluster member**: …SmartClipboard / ink /
**ArguMap**.

---

## Unblock trigger (operator)

When ready to ship:

1. Wire Apple Developer ID + notarization credentials.
2. **Decide Android posture.** Tauri mobile mipmap icons in tree
   suggest Android distribution ambition. Operator picks: ship
   macOS only first, or pursue Tauri 2 Android packaging
   simultaneously. Tauri 2 Android is still maturing — operator
   should verify the build path before committing to dual-platform
   v1.
3. Confirm SQLite migration story for existing users — auto-save
   means an installed user has local maps the upgrade must
   preserve.
4. Cut v2.0.0 release tag (v1 + v2 both in).
5. Verify signed/notarized DMG opens cleanly with no Gatekeeper
   warnings.

Estimated operator time once credentials are in hand: ~3 hours
including Android decision and notarization round-trip.

---

## Portfolio operating system instructions

| Aspect | Posture |
|---|---|
| Portfolio status | `Release Frozen` |
| Review cadence | Suspend overdue counting |
| Resurface conditions | (a) Apple signing credentials wired, (b) operator decides Android posture, or (c) operator opens a v2.1 scope packet |
| Co-batch with | Signing cluster: …SmartClipboard / ink / **ArguMap** — **now 21 repos** |
| Special concern | **Android packaging signals.** Tauri 2 mipmap icons are present without a confirmed mobile build path. Operator decision needed. |
| Special concern | **SQLite migration on upgrade.** Auto-save creates real user state — preserve through versioning. |

---

## Why this row has Android ambition

Most cluster members ship macOS-only initially. ArguMap has
`src-tauri/icons/android/mipmap-*/ic_launcher*.png` variants on
canonical main, which Tauri 2 generates only when targeting
Android. Two reads:

1. **Operator started exploring Tauri 2 Android** and the icons
   are leftover from the bootstrap. Drop the icons from v2 scope,
   ship macOS, revisit later.
2. **Operator intends dual-platform.** Then signing cluster posture
   adds an Android-side track: Play Console + Google sign-in
   credentials + AAB upload pipeline.

The PR for this disposition can't resolve the question — it's an
operator-only call. But this is worth surfacing before the macOS
signing round so the operator can plan.

---

## Reactivation procedure (for the next code session)

1. Verify `git branch -vv` shows `main` tracking `origin/main`.
   Already correct as of this disposition pass.
2. Review the local stash (`r10-argumap-stash` if created) for any
   uncommitted work.
3. Delete stale `codex/*` branches.
4. Re-run `pnpm install && pnpm tauri build` to confirm toolchain.
5. **Decide Android posture before signing round.**

---

## Last known reference

| Field | Value |
|---|---|
| `origin/main` tip | `72f4db3` (HEAD) |
| Last substantive commit | `c1bd868` feat: strength scoring (1-5) on nodes |
| Default branch | `main` |
| Build system | Tauri 2 + Rust + React + TypeScript + Vite + `@xyflow/react` + SQLite |
| Phases shipped | v1.0 (`f6f2e29`) + v2 (`911a752`) + strength scoring (`c1bd868`) |
| Release scaffolding | **None on `origin/main`** |
| Cross-platform signal | **Android mipmap icons in tree** — Tauri 2 mobile templates present |
| Blocker | Apple signing + Android posture decision + SQLite upgrade migration audit (operator-only) |
| Migration state | **No `legacy-origin` remote** — clean |
| Distinguishing feature | **Argument-mapping domain** — 4 node types + 4 edge types + structured templates is a richer schema than typical "graph editor" apps |
