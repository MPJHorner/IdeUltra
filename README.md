# IdeUltra

A snappy, native, local-first code IDE. Part of the **Ultra** family — alongside [MailboxUltra](https://mpjhorner.github.io/MailboxUltra/) and [PostbinUltra](https://mpjhorner.github.io/PostbinUltra/).

- Pure Rust + [egui](https://github.com/emilk/egui)
- Native macOS, single binary (~15 MB)
- Zero telemetry, MIT licensed
- Built for **speed** — startup under 150ms, keystroke-to-paint under 16ms

> **Status:** first usable MVP. Open a folder, browse files, edit, save. Syntax highlighting and find/replace are next. See [`plan.md`](./plan.md) for the deliverable-ordered roadmap.

## Shortcuts

| Key | Action |
|-|-|
| `⌘O` | Open File… |
| `⇧⌘O` | Open Folder… |
| `⌘S` | Save active tab |
| `⌘W` | Close active tab |
| `⌘[` / `⌘]` | Previous / next tab |
| `⌘1`…`⌘9` | Jump to tab N |

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
| D5  | Syntax highlighting | next |
| D6  | Find & replace | |
| D7  | Status bar & shortcuts | |
| D8  | Persistence | |
| D9  | File watching | |
| D10 | Packaging + docs site | |

## License

MIT
