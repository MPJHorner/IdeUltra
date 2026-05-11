# IdeUltra — MVP Build Plan

A native, snappy, local-first code IDE in the **Ultra** family (MailboxUltra, PostbinUltra). Pure Rust, single binary, no telemetry, MIT.

The brief: file browser on the left, text editor in the middle, the basics you'd expect. **Speed is the product.** Startup < 150ms, cold-open a 1MB file < 50ms, keystroke-to-paint < 16ms. If a feature jeopardises that, it's cut.

---



## 1. Goals and Non-Goals

### Goals (MVP)
- Open a folder, browse its files in a tree, click to open in editor.
- Edit plaintext / code with syntax highlighting and a monospace font.
- Multiple open files via tabs.
- Find / replace within the current file.
- Save (Cmd+S), Save As, Reload-from-disk.
- Native macOS app (universal binary, ~10–15 MB), launches under 150ms.
- Persist window position, last folder, open tabs, theme.
- GitHub Pages docs site matching the Ultra house style.

### Non-Goals (explicitly out of MVP)
- LSP / language servers / IntelliSense.
- Integrated terminal.
- Git integration.
- Multi-cursor, vim mode, command palette plugins.
- Debugger, build tasks, project-wide search/replace.
- Extensions / plugins.
- Windows / Linux builds (Rust is portable so it'll likely run; we only *test* and *ship* macOS for MVP).

Each non-goal is a candidate for a follow-up `v0.2` and should not influence MVP architecture except to **not paint us into a corner**.

---

## 2. Tech Stack

| Concern              | Choice                                | Why |
|----------------------|---------------------------------------|-----|
| Language             | Rust (edition 2021, MSRV 1.78)        | Matches MailboxUltra / PostbinUltra. Single binary, fast, safe. |
| GUI framework        | `eframe` + `egui` (latest stable)     | Immediate-mode, native, fast cold start, zero web stack. |
| Code editor widget   | `egui_code_editor`                    | Drop-in multiline editor with syntax theming. We'll fork/vendor if needed. |
| Syntax highlighting  | `syntect` (via `egui_code_editor` or direct) | Sublime-grade rules; bundled themes. |
| File watching        | `notify` (debounced)                  | Detect external file changes for "reload?" prompt. |
| Persistence          | `serde` + `directories` crate, JSON on disk | App state in `~/Library/Application Support/IdeUltra/state.json`. |
| Logging              | `tracing` + NDJSON to file            | Same shape as MailboxUltra's NDJSON for agent-friendliness. |
| Packaging            | `cargo-bundle` → `.app`, `create-dmg` → `.dmg` | Same path as the other Ultra apps. |
| Docs site            | GitHub Pages, static HTML/CSS in `/docs` | Match the Ultra navigation: Install / Quick Start / Preferences / Changelog. |

**Crates explicitly considered and rejected:**
- `tauri` — webview-based, violates the snappy / native premise.
- `iced` — viable, but egui has more mature code-editing primitives for our scope.
- `tree-sitter` — heavier than `syntect`; revisit in v0.2 if we need LSP-grade highlighting.

---

## 3. Architecture (one-screen overview)

```
                        ┌──────────────────────────────────────────┐
                        │           IdeUltra (eframe::App)         │
                        ├──────────────────────────────────────────┤
                        │  AppState                                │
                        │   ├─ workspace: Option<Workspace>        │
                        │   │   └─ root_path, file_tree, watcher   │
                        │   ├─ editors: Vec<EditorTab>             │
                        │   │   └─ path, buffer, dirty, scroll,    │
                        │   │      selection, undo_stack, lang     │
                        │   ├─ active_tab: usize                   │
                        │   ├─ find: FindState                     │
                        │   ├─ settings: Settings (persisted)      │
                        │   └─ ui: UiState (transient)             │
                        └──────────────────────────────────────────┘
                                       │ update() each frame
                                       ▼
   ┌─────────────┬───────────────────────────────────────────────┐
   │ left panel  │ central panel                                 │
   │ file tree   │ ┌───── tab strip ────────────────────────────┐│
   │ (egui Tree) │ │ main.rs ● │ Cargo.toml │ README.md         ││
   │             │ ├────────────────────────────────────────────┤│
   │             │ │ CodeEditor (egui_code_editor)              ││
   │             │ │                                            ││
   │             │ └────────────────────────────────────────────┘│
   │             │ find/replace bar (collapsible)                │
   ├─────────────┴───────────────────────────────────────────────┤
   │ status bar: path · ln/col · LF · UTF-8 · 142 KB · Rust       │
   └─────────────────────────────────────────────────────────────┘
```

**Module layout**
```
src/
├─ main.rs                # eframe entrypoint
├─ app.rs                 # AppState + impl eframe::App (update loop)
├─ workspace/
│   ├─ mod.rs             # Workspace, open_folder()
│   ├─ tree.rs            # FileTree (lazy, sorted, gitignore-aware)
│   └─ watcher.rs         # notify integration, debounced events
├─ editor/
│   ├─ mod.rs             # EditorTab, open/save/reload
│   ├─ buffer.rs          # text buffer wrapper, dirty tracking
│   └─ language.rs        # extension → syntect syntax mapping
├─ ui/
│   ├─ sidebar.rs         # left panel: file tree rendering
│   ├─ tabs.rs            # tab strip
│   ├─ editor_panel.rs    # central editor area
│   ├─ find_bar.rs        # find / replace
│   ├─ status_bar.rs      # bottom bar
│   └─ shortcuts.rs       # keymap → action dispatch
├─ settings.rs            # serde-backed settings + load/save
├─ persistence.rs         # window state, last folder, open tabs
└─ logging.rs             # tracing + NDJSON sink
```

---

## 4. Deliverables (ordered)

Each deliverable is a **shippable increment**: it ends with the app building, running, and demonstrably more useful than before. Tag a git commit at the end of each (`mvp-d1`, `mvp-d2`, …) so we can bisect regressions.

The deliverables are sized to land in roughly one focused session each. If a deliverable starts ballooning, that's the signal to split it, not to keep going.

---

### D1 — Project scaffold and empty native window
**Outcome:** `cargo run` opens a 1200×800 native macOS window titled "IdeUltra" with a menu bar and empty central panel. Cold start measured and recorded.

**Tasks**
1. `cargo init --bin ideultra` in `/Users/matt/GitHub/IdeUltra/`.
2. Add `Cargo.toml` deps: `eframe = "0.28"`, `egui = "0.28"`, `serde`, `serde_json`, `directories`, `tracing`, `tracing-subscriber`, `anyhow`.
3. `src/main.rs`: `eframe::run_native()` with `NativeOptions { viewport: ViewportBuilder::default().with_inner_size([1200., 800.]).with_min_inner_size([640., 400.]).with_title("IdeUltra"), ..Default::default() }`.
4. `src/app.rs`: `IdeUltraApp` struct, empty `impl eframe::App` with `update()` rendering `CentralPanel` containing a placeholder label.
5. `src/logging.rs`: init `tracing_subscriber` with env filter + NDJSON file appender under `~/Library/Application Support/IdeUltra/log.ndjson`.
6. README.md with one paragraph + screenshot placeholder.
7. `.gitignore` for `target/`, `*.dmg`, `dist/`.

**Acceptance criteria**
- `cargo run --release` cold-starts in < 200ms on M-series Mac (measure with `time`).
- Binary size < 20 MB stripped.
- No panics on close. Window remembers nothing yet — that's D8.

---

### D2 — File browser sidebar (read-only)
**Outcome:** "File → Open Folder…" prompts for a directory; the left panel shows a collapsible tree. Clicking a file does **nothing yet** (D3 wires it up). The tree handles 10k+ files without hitching.

**Tasks**
1. `workspace/mod.rs`: `Workspace { root: PathBuf, tree: FileTree }`. `open_folder(path)` constructor.
2. `workspace/tree.rs`: `FileTree` node = `{ name, path, is_dir, children: Option<Vec<Node>> }`. Children loaded **lazily** on first expand (don't walk the whole tree upfront).
3. Sort: directories first, then files, both case-insensitive alphabetic.
4. Respect `.gitignore` at the root via `ignore` crate (single-file simple parser is enough; full crate is fine too, it's small).
5. `ui/sidebar.rs`: render with `egui::CollapsingHeader` per directory. Use the `egui_extras` icon set or text glyphs (▸/▾, 📁/📄) — no PNG assets.
6. Menu bar: `File → Open Folder…` opens native folder picker via `rfd` crate.
7. Drag the splitter between sidebar and central panel; persist the width to `UiState` (in-memory; persisted in D8).

**Acceptance criteria**
- Opens `/Users/matt/GitHub` (which has 80+ subdirs) without visible lag.
- Expanding a 5k-file directory paints under one frame.
- `.git/`, `node_modules/`, `target/` are hidden when gitignored.
- Splitter drag is smooth (60fps).

---

### D3 — Editor core: open one file, edit, save
**Outcome:** Click a file in the tree → it loads into a `egui::TextEdit::multiline` taking the central panel. Cmd+S writes back to disk. Dirty state tracked. **Plaintext only**, no highlighting yet.

**Tasks**
1. `editor/buffer.rs`: `Buffer { text: String, dirty: bool, original_hash: u64 }`. `is_dirty()` compares hash.
2. `editor/mod.rs`: `EditorTab { path, buffer, scroll_offset, last_loaded_mtime }`. `open(path)`, `save()`, `reload()`.
3. Read file with `std::fs::read_to_string` (UTF-8 only for MVP; show error toast for non-UTF-8 — that's a v0.2 problem).
4. `ui/editor_panel.rs`: monospace font (load JetBrains Mono via `egui::FontDefinitions`), tab width 4 visually, soft-wrap off by default.
5. Shortcut: Cmd+S → save active tab. Cmd+O → open file picker (independent of folder open). Cmd+W → close active tab (prompt if dirty).
6. Window title reflects `<filename> — IdeUltra`, with `●` prefix when dirty.
7. Confirm-on-quit if any tab dirty.

**Acceptance criteria**
- Open a 1 MB source file: ready-to-type in under 50ms.
- Typing a character: visible next frame (< 16ms). Measure with `egui` debug overlay.
- Editing then saving writes exactly the bytes shown (no trailing newline mangling, no BOM injection).
- Closing dirty tab without saving shows a native confirm dialog.

---

### D4 — Tabs (multiple open files)
**Outcome:** Opening a second file adds a tab rather than replacing. Click tabs to switch. Middle-click or × to close. Cmd+1..9 jumps to tab N. Cmd+T does nothing yet (reserved for command palette).

**Tasks**
1. `AppState.editors: Vec<EditorTab>`, `active_tab: usize`.
2. `ui/tabs.rs`: horizontal scroll strip above editor. Each tab shows `filename` + dirty dot + close ×.
3. Switching tabs preserves scroll position and selection per-tab.
4. Cmd+Shift+[ / ] cycle tabs. Cmd+1..9 absolute. Cmd+W closes active.
5. Re-opening an already-open file just focuses the existing tab.
6. Drag-reorder tabs (egui supports this with `egui_dnd` — only add if it stays under 30 lines, otherwise defer to v0.2).

**Acceptance criteria**
- Open 20 files: tab strip scrolls smoothly, no per-frame allocations spike (check with debug overlay).
- Switching tabs is instant (< 16ms, no re-tokenisation yet — that comes in D5).
- Closing a dirty tab prompts; closing a clean tab is silent.

---

### D5 — Syntax highlighting
**Outcome:** Common languages get coloured (Rust, JS/TS, Python, Go, Ruby, HTML, CSS, JSON, YAML, TOML, Markdown, Shell, SQL). Theme matches a built-in dark and light from `syntect`'s defaults.

**Tasks**
1. Add `syntect` (with default features off, only `parsing` + `default-syntaxes` + `default-themes`).
2. `editor/language.rs`: extension → `SyntaxReference` map, cached.
3. Replace `TextEdit::multiline` with `egui_code_editor::CodeEditor` configured with `syntect` highlighting. Test that performance is still in budget.
4. Cache highlighted spans per-line; invalidate only edited lines (lazy: invalidate whole buffer on edit for MVP if line-level proves fiddly — measure first).
5. Theme picker in settings: `Dark`, `Light`, `Follow System`. Hook into `egui::Context::set_visuals`.
6. Status bar shows detected language (D7 builds the bar; for now a corner label is fine).

**Acceptance criteria**
- A 5,000-line Rust file scrolls at 60fps.
- Typing in a 1k-line file: still < 16ms keystroke-to-paint.
- Unknown extension falls back to plaintext with no error.
- Theme switch is instant, no flicker.

**Risk:** `egui_code_editor` may be too minimal — if highlighting is slow or buggy, fall back to `TextEdit::multiline` + a custom `LayoutJob` built from `syntect` tokens. Time-box the integration to 3 hours; if it's not working, switch approach the same day.

---

### D6 — Find and replace (current file)
**Outcome:** Cmd+F opens a find bar above the editor. Cmd+Alt+F opens find+replace. Enter / Shift+Enter jump next/prev. Esc closes. Highlights all matches; current match is brighter.

**Tasks**
1. `ui/find_bar.rs`: collapsible bar at top of editor panel. Inputs: query, options (case-sensitive, whole-word, regex).
2. `FindState { query, options, matches: Vec<Range<usize>>, current: usize }`. Re-compute matches on query or buffer change (debounce 50ms).
3. Render matches with a background highlight via `egui_code_editor` custom layout, or as overlay rects if the widget can't.
4. Replace: replace current match; Replace All; both push to undo stack as single edits.
5. Regex via `regex` crate; invalid regex shows red border on input, no crash.

**Acceptance criteria**
- Find across 10k-line file completes in < 50ms.
- Replace All on 1,000 matches is one undo step.
- Esc closes the bar and returns focus to the editor with cursor preserved.

---

### D7 — Status bar and command shortcuts
**Outcome:** Bottom status bar shows: full path · line/col · selection length · line endings (LF/CRLF) · encoding (UTF-8) · file size · detected language. All major actions are keyboard-driven; menu mirrors them.

**Tasks**
1. `ui/status_bar.rs`: `egui::TopBottomPanel::bottom`. Each segment clickable where it makes sense (line/col → "Go to line" prompt).
2. `ui/shortcuts.rs`: centralise keymap. Cmd+P reserved (no-op placeholder for future file-fuzzy-find). Cmd+G "Go to line".
3. Menu bar entries: File (Open Folder, Open File, Save, Save As, Close Tab, Quit), Edit (Undo, Redo, Cut, Copy, Paste, Find, Replace), View (Toggle Sidebar, Theme submenu, Zoom +/-/Reset), Help (Documentation → opens docs URL).
4. Zoom: Cmd+= / Cmd+- adjusts `egui::Context::set_zoom_factor`.

**Acceptance criteria**
- Every menu item has a working shortcut shown next to it.
- "Go to line" works on a 100k-line file instantly.
- Toggling sidebar with Cmd+B animates (or snaps — don't over-engineer) without reflow jank.

---

### D8 — Persistence: window, tabs, settings
**Outcome:** Quit and relaunch: same window position and size, same folder open, same tabs restored in order, same active tab, same theme and sidebar width. Settings file is human-readable JSON.

**Tasks**
1. `settings.rs`: `Settings { theme, sidebar_width, zoom, font_size, tab_width, soft_wrap }`. Default impl.
2. `persistence.rs`: `SessionState { window: WindowState, last_folder: Option<PathBuf>, open_tabs: Vec<TabState>, active_tab: usize }`. Saved on every meaningful change, debounced 500ms.
3. Path: `~/Library/Application Support/IdeUltra/state.json` and `settings.json` via `directories::ProjectDirs`.
4. On startup: load both; if folder no longer exists, start empty; if a tab's file no longer exists, skip it with a warn log.
5. Unsaved buffers are **not** restored in MVP (path-only restoration). Document this in the changelog. Auto-save / crash recovery is v0.2.

**Acceptance criteria**
- Force-quit, relaunch: full session restored in < 200ms.
- Settings file edited by hand and saved while the app is open: app picks up the change next launch (no need to live-reload in MVP).
- Deleting a tracked file externally: the tab is dropped on next launch, no crash.

---

### D9 — File watching and external change handling
**Outcome:** When a file changes on disk (e.g. `git checkout`, formatter run), the editor either auto-reloads (clean buffer) or shows a non-modal banner: "This file changed on disk. Reload / Keep mine / Diff".

**Tasks**
1. `workspace/watcher.rs`: `notify::RecommendedWatcher` watching the workspace root recursively, debounced 100ms via `notify-debouncer-mini`.
2. Events → `AppState` via `crossbeam_channel` polled each `update()`. Update file tree on create/delete/rename. Mark affected `EditorTab` with `external_change: true`.
3. Clean tab: silently reload, restore scroll/cursor.
4. Dirty tab: banner inside the editor panel (not modal — never block).
5. "Diff" button is a v0.2 placeholder for now (greyed out, tooltip "Coming in v0.2").

**Acceptance criteria**
- `touch file.rs` while it's open and clean → reloads silently.
- Edit on disk while dirty → banner appears, editor still usable.
- Renaming a watched file updates the tab title and the tree.
- Watcher does not pin CPU above 0.5% idle.

---

### D10 — Packaging, app icon, and docs site
**Outcome:** `make release` produces `IdeUltra.app` and `IdeUltra-0.1.0.dmg`. GitHub Pages site at `mpjhorner.github.io/IdeUltra` matches the Ultra family layout.

**Tasks**

**Packaging**
1. `cargo-bundle` config in `Cargo.toml` (`[package.metadata.bundle]`): identifier `com.mpjhorner.ideultra`, category `public.app-category.developer-tools`, icon `assets/icon.icns`.
2. Universal binary: `cargo build --release --target aarch64-apple-darwin && cargo build --release --target x86_64-apple-darwin && lipo -create -output …`.
3. `create-dmg` script in `scripts/package.sh`. Output to `dist/`.
4. App icon: simple typographic glyph (e.g. `</>` or `▸`) in the Ultra family palette. SVG → `icon.icns` via `iconutil`.
5. Codesigning + notarisation: **out of MVP** unless certs are already in place. Document the unsigned-app `xattr -d com.apple.quarantine` workaround in the install page (MailboxUltra does this).

**Docs site (`/docs` directory, GitHub Pages from `main` branch)**
6. `docs/index.html`: hero (one-line install, screenshot, "What it is" paragraph), feature grid, 30-second tour, footer with full doc links — match MailboxUltra structure exactly.
7. `docs/install.md`: brew tap if applicable, manual `.dmg` install, quarantine note.
8. `docs/quickstart.md`: open a folder, edit a file, find/replace, tabs, shortcuts cheat-sheet.
9. `docs/preferences.md`: settings.json schema, paths, theme list.
10. `docs/shortcuts.md`: full keymap table.
11. `docs/changelog.md` + RSS feed (`docs/changelog.xml`) — mirror PostbinUltra.
12. CSS shared with the Ultra family if a stylesheet already exists; otherwise lift the structure from MailboxUltra's `index.html` and adapt copy.
13. `docs/use-cases.md`: "quick code edits without booting VS Code", "reviewing a downloaded repo", "agent-friendly editor that doesn't lag SSH'd sessions", etc.

**Acceptance criteria**
- `.dmg` mounts, dragging to Applications works, first launch shows the empty window in < 200ms.
- Docs site renders correctly on mobile (matching the Ultra responsive style).
- Every footer link works; changelog has at least one entry (`v0.1.0 — Initial MVP release`).
- A new user can go from landing page → installed → editing a file in under 60 seconds following only the Quick Start.

---

## 5. Performance Budget (enforced through D1–D10)

These are not aspirational — every deliverable's acceptance criteria reference them. If a deliverable breaks budget, fix it before tagging.

| Metric                              | Budget        | How measured |
|-------------------------------------|---------------|--------------|
| Cold start (release, M-series)      | < 150ms       | `time ./IdeUltra.app/Contents/MacOS/ideultra --measure-start` |
| Open 1 MB file                      | < 50ms        | Instrumented log line in `EditorTab::open` |
| Keystroke → paint                   | < 16ms        | `egui` debug overlay frame time |
| Scroll 10k-line file                | 60fps         | Visual + frame time overlay |
| Idle CPU                            | < 0.5%        | macOS Activity Monitor, 10s sample |
| Binary size (universal, stripped)   | < 20 MB       | `du -h IdeUltra.app/Contents/MacOS/ideultra` |
| Memory at rest with 5 tabs open     | < 80 MB       | Activity Monitor |

---

## 6. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `egui_code_editor` is too minimal | Medium | High (D5) | Time-box to 3h. Fallback: hand-rolled `LayoutJob` from `syntect` tokens over `TextEdit::multiline`. |
| Syntax highlighting is the perf bottleneck | Medium | High | Per-line cache; lazy invalidation. Worst case: highlight only the viewport. |
| `notify` is noisy on macOS (FSEvents) | Medium | Medium (D9) | Use `notify-debouncer-mini`. Ignore events under `.git/`, `target/`, `node_modules/`. |
| Universal binary build is fiddly | Low | Low (D10) | Document the two-target + `lipo` recipe. GitHub Actions builds both arches if local setup is annoying. |
| Scope creep ("just add a terminal") | High | High | Non-Goals list is the gate. Anything in it is `v0.2`, full stop. |

---

## 7. v0.2 Backlog (post-MVP, not part of this plan)

Listed only so we don't accidentally do them now: command palette · fuzzy file finder · project-wide search · multi-cursor · vim mode · git gutter · integrated terminal · LSP · auto-save / crash recovery · Linux & Windows builds · diff view for external changes · markdown preview · session snapshots.

---

## 8. Definition of Done for MVP

- All ten deliverables merged and tagged.
- All performance budgets met on a clean M-series Mac.
- Docs site live at `mpjhorner.github.io/IdeUltra` with all sections populated.
- `v0.1.0` tag pushed; `.dmg` attached to the GitHub release.
- One paragraph posted to the Ultra family README cross-linking to the new app.
