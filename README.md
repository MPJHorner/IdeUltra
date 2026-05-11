# IdeUltra

A snappy, native, local-first code IDE. Part of the **Ultra** family — alongside [MailboxUltra](https://mpjhorner.github.io/MailboxUltra/) and [PostbinUltra](https://mpjhorner.github.io/PostbinUltra/).

- Pure Rust + [egui](https://github.com/emilk/egui)
- Native macOS, single binary (~15 MB)
- Zero telemetry, MIT licensed
- Built for **speed** — startup under 150ms, keystroke-to-paint under 16ms

> **Status:** v0.1.0 — feature-complete MVP. All ten deliverables shipped. Docs at <https://mpjhorner.github.io/IdeUltra/>. See [`plan.md`](./plan.md) for the deliverable-ordered build log.

## Shortcuts

| Key | Action |
|-|-|
| `⌘O` | Open File… |
| `⇧⌘O` | Open Folder… |
| `⌘S` | Save active tab |
| `⌘W` | Close active tab |
| `⌘[` / `⌘]` | Previous / next tab |
| `⌘1`…`⌘9` | Jump to tab N |
| `⌘F` | Find |
| `⌥⌘F` | Find & Replace |
| `⇧⌘F` | Search in project |
| `Enter` / `⇧Enter` | Next / previous match |
| `Esc` | Close find bar |
| `⌘P` | Go to file (fuzzy finder) |
| `⇧⌘P` | Command palette |
| `⌘G` | Go to line |
| `⌘B` | Toggle sidebar |
| `⌘=` / `⌘-` / `⌘0` | Zoom in / out / reset |

## Tests

```bash
cargo test
```

71 unit tests covering find/replace logic, buffer dirty tracking, language detection, file-tree behaviour (lazy load, sorting, gitignore, invalidate-on-change), line/column conversion, line-ending detection, settings/session persistence (round-trip, atomic writes, missing-tab cleanup, debounce), fuzzy file finder scoring + indexing, project-wide search (line numbers, binary detection, multi-file aggregation, regex errors), and the command palette registry.

## State files

IdeUltra writes two human-readable JSON files to
`~/Library/Application Support/com.mpjhorner.IdeUltra/`:

- `settings.json` — theme, zoom, sidebar state
- `session.json` — window size/position, last folder, open tabs, active tab

Edit them by hand and they'll be picked up on next launch.

## Run from source

```bash
git clone https://github.com/MPJHorner/IdeUltra.git
cd IdeUltra
cargo run --release
```

## Build a .app and .dmg

```bash
./scripts/package.sh 0.1.0
# → dist/IdeUltra.app
# → dist/IdeUltra-0.1.0.dmg
```

arm64 only for v0.1.0; universal binary (Intel + arm64) is planned for v0.1.1.

## Roadmap

The MVP ships in 10 deliverables (D1–D10). Tracked in [`plan.md`](./plan.md).

| | Deliverable | Status |
|-|-|-|
| D1  | Scaffold + empty native window | shipped |
| D2  | File browser sidebar | shipped |
| D3  | Editor core (open / edit / save) | shipped |
| D4  | Tabs | shipped |
| D5  | Syntax highlighting | shipped |
| D6  | Find & replace | shipped |
| D7  | Status bar & shortcuts | shipped |
| D8  | Persistence | shipped |
| D9  | File watching | shipped |
| D10 | Packaging + docs site | shipped |

**MVP complete.** Tagged `v0.1.0`. Docs live at <https://mpjhorner.github.io/IdeUltra/>.

### v0.2 (in progress)

| | Feature | Status |
|-|-|-|
| 1 | Fuzzy file finder (⌘P) | shipped on `main` |
| 2 | Project-wide search (⇧⌘F) | shipped on `main` |
| 3 | Command palette (⇧⌘P) | shipped on `main` |

## License

MIT
