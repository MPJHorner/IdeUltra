# IdeUltra

A snappy, native, local-first code IDE. Part of the **Ultra** family — alongside [MailboxUltra](https://mpjhorner.github.io/MailboxUltra/) and [PostbinUltra](https://mpjhorner.github.io/PostbinUltra/).

- Pure Rust + [egui](https://github.com/emilk/egui)
- Native macOS, single binary (~15 MB)
- Zero telemetry, MIT licensed
- Built for **speed** — startup under 150ms, keystroke-to-paint under 16ms

> **Status:** feature-complete MVP. Open a folder, browse, edit with syntax highlighting, find/replace, go-to-line. Tabs, theme, zoom, window and folder all persist. External file changes propagate live — clean buffers reload silently, dirty buffers show a "Reload / Keep mine" banner. D10 (packaging + docs site) is next. See [`plan.md`](./plan.md).

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
| `Enter` / `⇧Enter` | Next / previous match |
| `Esc` | Close find bar |
| `⌘G` | Go to line |
| `⌘B` | Toggle sidebar |
| `⌘=` / `⌘-` / `⌘0` | Zoom in / out / reset |

## Tests

```bash
cargo test
```

43 unit tests covering find/replace logic, buffer dirty tracking, language detection, file-tree behaviour (lazy load, sorting, gitignore, invalidate-on-change), line/column conversion, line-ending detection, and settings/session persistence (round-trip, atomic writes, missing-tab cleanup, debounce).

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
| D10 | Packaging + docs site | next |

## License

MIT
