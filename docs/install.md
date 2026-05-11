---
title: Install
permalink: /install.html
---

IdeUltra is a single ~7 MB native binary. No installer, no daemon, no helper services.

## From source (recommended for v0.1)

You need Rust 1.78 or newer. [Install Rust](https://rustup.rs) if you don't have it.

```bash
git clone https://github.com/MPJHorner/IdeUltra.git
cd IdeUltra
cargo run --release
```

The first build takes about 30s. Subsequent builds are incremental.

## Pre-built `.dmg`

Pre-built binaries are attached to each [GitHub release](https://github.com/MPJHorner/IdeUltra/releases).
Download the `.dmg`, drag `IdeUltra.app` into `Applications`.

**First launch:** because the v0.1.0 binary isn't notarised, macOS will
refuse to open it. Either right-click the app in Finder and choose
**Open** (you'll get a one-time prompt), or remove the quarantine flag:

```bash
xattr -d com.apple.quarantine /Applications/IdeUltra.app
```

## What it puts on your machine

| Where | What |
|-|-|
| The `.app` bundle | The binary and `Info.plist`. ~7 MB. |
| `~/Library/Application Support/com.mpjhorner.IdeUltra/settings.json` | Theme, zoom, sidebar state. Human-readable JSON. |
| `~/Library/Application Support/com.mpjhorner.IdeUltra/session.json` | Window position, last folder, open tabs. |
| `~/Library/Application Support/com.mpjhorner.IdeUltra/log.ndjson` | Structured NDJSON log. Append-only. |

Nothing else. No background daemons. No analytics endpoints.

## Uninstalling

```bash
rm -rf /Applications/IdeUltra.app
rm -rf ~/Library/Application\ Support/com.mpjhorner.IdeUltra
```
