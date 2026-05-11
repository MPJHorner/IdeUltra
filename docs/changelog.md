---
title: Changelog
permalink: /changelog.html
---

## v0.3.0 — 2026-05-11

- **Markdown preview (⌥⌘M).** When the active tab is a `.md` / `.markdown` /
  `.mdx` file, a live-rendered preview appears in a resizable right side-panel.
  Supports headings, paragraphs, bold, italic, inline code, fenced code blocks,
  bullet and numbered lists, blockquotes, links (click to open), and horizontal
  rules. Preview-visibility is persisted in `settings.json`.
- **Diff view for external changes.** The amber "this file changed on disk"
  banner now has a **View diff** button. It opens a unified line diff with
  red removals and green additions, showing both the old and new line numbers.
  Reload or Keep mine are available directly from the modal.
- **Recent files.** `File → Open Recent` shows the last 10 opened files
  with home-relative paths. The full history (up to 30 entries, deduped,
  most-recent-first) persists in `session.json`. **Clear Recent** wipes it.
  Stale entries whose files have been moved or deleted are pruned on launch.

## v0.2.0 — 2026-05-11

- **Fuzzy file finder (⌘P).** Type to filter files across the workspace.
  Arrow keys navigate, Enter opens, Esc closes. Up to 50,000 files indexed
  per workspace; results re-rank on every keystroke. Word-boundary and
  prefix matches outscore arbitrary subsequence matches.
- **Project-wide search (⇧⌘F).** Search a query across every file in the
  workspace. Results are grouped by file with the matching line and 1-based
  line number; click a line to open the file and jump to the match. Reuses
  the find bar's case / whole-word / regex toggles. Files larger than 1 MB
  and binary files are skipped. Capped at 500 results to keep the UI snappy.
- **Command palette (⇧⌘P).** A single fuzzy-searchable list of every menu
  action — File, Edit, Go, View. Reuses the fuzzy file finder's scorer so
  partial queries like "theme dark" rank the right command first. Adding a
  new command is a compile error anywhere that doesn't handle it.
- **Crash recovery for dirty buffers.** Every dirty buffer gets a snapshot
  written to `~/Library/Application Support/com.mpjhorner.IdeUltra/recovery/`
  on a 1-second debounce. On normal save the snapshot is dropped. After a
  crash or force-quit, the next launch shows a modal listing the unsaved
  buffers and offers **Restore all**, **Discard all**, or **Decide later**.
  Recoveries whose original file has been saved through some other channel
  (mtime is newer than the recovery) are silently cleaned up on scan.
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
