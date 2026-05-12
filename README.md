# IdeUltra

A snappy, native, local-first code IDE. **2026 luxury light** — white
canvas, deep-ink accent, generous whitespace. Pure Rust + [egui](https://github.com/emilk/egui).

Part of the **Ultra** family — alongside
[MailboxUltra](https://mpjhorner.github.io/MailboxUltra/) and
[PostbinUltra](https://mpjhorner.github.io/PostbinUltra/).

- Native macOS · universal binary (~16 MB)
- 239 unit tests, all green
- Zero telemetry · MIT licensed
- Cold start < 150 ms · keystroke-to-paint < 16 ms

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/MPJHorner/IdeUltra/main/scripts/install.sh | bash
```

That fetches the latest universal `.dmg` from GitHub, copies
`IdeUltra.app` into `/Applications`, and clears the Gatekeeper
quarantine flag. macOS 11+, Apple Silicon or Intel.

Prefer to do it by hand? Grab the latest DMG from the
[releases page](https://github.com/MPJHorner/IdeUltra/releases/latest)
and drag the app into `/Applications`. First launch will be blocked
by Gatekeeper (the binary isn't notarised yet); run:

```bash
xattr -d com.apple.quarantine /Applications/IdeUltra.app
```

## Run from source

```bash
git clone https://github.com/MPJHorner/IdeUltra.git
cd IdeUltra
cargo run --release
```

Rust 1.78+ required. First build ~30 s; incremental builds are
sub-second.

## Build a `.app` and `.dmg`

```bash
./scripts/package.sh 0.23.1
# → dist/IdeUltra.app          (universal arm64 + x86_64)
# → dist/IdeUltra-0.23.1.dmg
```

Set `IDEULTRA_ARCH=arm64` for a slim, single-arch build.

## Shortcuts

| Key                       | Action                                       |
|---------------------------|----------------------------------------------|
| `⌘N`                      | New file (Untitled)                          |
| `⌘O` / `⇧⌘O`              | Open file / Open folder                      |
| `⌘S` / `⇧⌘S`              | Save / Save As                               |
| `⌘W`                      | Close tab (prompts on dirty)                 |
| `⌘[` / `⌘]`               | Previous / next tab                          |
| `⌃Tab`                    | Quick-switch most-recently-used tabs         |
| `⌘1` … `⌘9`               | Jump to tab N                                |
| `⌘P` / `⇧⌘P`              | Go to file / Command palette                 |
| `⌘D`                      | Select word at cursor / next occurrence      |
| `⌘F` / `⌥⌘F` / `⇧⌘F`      | Find / Find & Replace / Find in project      |
| `⌘/`                      | Toggle line comment                          |
| `Tab` / `⇧Tab`            | Indent / dedent selection                    |
| `⌘G` / `⌘B` / `⌘\`        | Go to line / Toggle sidebar / Split editor   |
| `⌥⌘M` / `⌘,`              | Toggle markdown preview / Preferences        |
| `⌘=` / `⌘-` / `⌘0`        | Zoom in / out / reset                        |

Default preset; pick **VS Code** or **PhpStorm** from
`Edit → Keymap` for the other conventions.

## Tests

```bash
cargo test
```

239 unit tests across find/replace, buffer state, language detection,
file-tree behaviour, fuzzy search scoring, project-wide search,
command palette registry, crash recovery, markdown parsing, line-diff,
recent-files MRU, text transforms, git porcelain parsing, brackets,
auto-pair, line/col conversion, line-ending detection, settings/session
persistence, name validation, indent / dedent, version compare for the
updater, and select-next-occurrence.

## Design

See [`STYLE_GUIDE.md`](./STYLE_GUIDE.md) — distilled from Zed, Linear,
Vercel Geist, Stripe docs, JetBrains Fleet, shadcn/ui. The aesthetic is
"luxury light": white paper, deep-ink accent, almost-invisible chrome.
Code carries the colour; chrome stays out of the way.

## State files

`~/Library/Application Support/com.mpjhorner.IdeUltra/`:

- `settings.json` — theme, zoom, keymap, soft-wrap, autosave toggles
- `session.json` — window, open tabs, split state, recent files & folders
- `recovery/<hash>.{json,txt}` — crash-recovery snapshots, surfaced on next launch
- `log.ndjson` — structured tracing output

All human-readable, all atomic writes.

## License

MIT
