---
title: Changelog
permalink: /changelog.html
---

## v0.2.0 — unreleased (on `main`)

- **Fuzzy file finder (⌘P).** Type to filter files across the workspace.
  Arrow keys navigate, Enter opens, Esc closes. Up to 50,000 files indexed
  per workspace; results re-rank on every keystroke. Word-boundary and
  prefix matches outscore arbitrary subsequence matches.
- Index auto-invalidates when the file watcher sees creates / removes /
  renames, so quickly-cloned repos always see their latest shape.

## v0.1.0 — initial MVP

Ten deliverables (D1–D10), all shipped. 43 unit tests, all green.
Native macOS, arm64. Universal binary in v0.1.1.

**Editor**

- File tree on the left, tabs, monospace editor in the centre.
- Syntax highlighting for 100+ languages via `syntect`.
- Dark and light themes (View → Theme).
- Find &amp; replace with case-sensitive / whole-word / regex toggles.
- Go to line (<code>⌘G</code>), sidebar toggle (<code>⌘B</code>), zoom (<code>⌘=</code>/<code>⌘-</code>/<code>⌘0</code>).
- Status bar: path, line/col, line endings, encoding, byte count, detected language.

**Workspace**

- Lazy, gitignore-aware file tree. `.git`, `target`, `node_modules` always hidden.
- Live file watcher: clean buffers reload silently, dirty buffers show a "Reload / Keep mine" banner.

**Persistence**

- Window size and position, sidebar state, theme, zoom, last folder, open tabs and active tab all survive a relaunch.
- Atomic writes (`.tmp` + rename), debounced 500ms.
- State files are human-readable JSON. Hand-edit them at will.

**Footprint**

- ~7 MB arm64 binary.
- Cold start &lt; 150ms.
- Zero telemetry. No background services. No sign-in.

## Not yet (the v0.2 backlog)

- Universal (Intel + arm64) binary, code-signed and notarised.
- Command palette and fuzzy file finder.
- Project-wide search and replace.
- Vim mode, multi-cursor, git gutter.
- Integrated terminal.
- LSP / language servers.
- Auto-save and crash recovery for dirty buffers.
- Diff view for external changes.
- Linux and Windows builds.
