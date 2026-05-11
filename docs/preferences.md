---
title: Preferences
permalink: /preferences.html
---

IdeUltra has no preferences pane — everything you can change is either a
menu item or a value in a small JSON file. Both live in
`~/Library/Application Support/com.mpjhorner.IdeUltra/`.

## `settings.json`

User-tunable preferences. Survive across sessions, never touched during normal editing.

```json
{
  "theme": "Dark",
  "zoom": 1.0,
  "sidebar_width": 260.0,
  "sidebar_visible": true
}
```

| Key | Type | Notes |
|-|-|-|
| `theme` | `"Dark"` or `"Light"` | Use **View → Theme** in the app. |
| `zoom` | float, 0.5–3.0 | Use <code>⌘=</code> / <code>⌘-</code> / <code>⌘0</code>. |
| `sidebar_width` | float, ≥160 | Drag the splitter. |
| `sidebar_visible` | bool | <code>⌘B</code> toggles. |

## `session.json`

What you had open when you last quit. Restored on relaunch.

```json
{
  "window": { "size": [1400, 900], "pos": [200, 100] },
  "last_folder": "/Users/you/work/myproject",
  "open_tabs": [
    "/Users/you/work/myproject/src/main.rs",
    "/Users/you/work/myproject/Cargo.toml"
  ],
  "active_tab": 1
}
```

Hand-edit either file (close IdeUltra first) and the next launch picks up the change. Both files are written atomically (`.tmp` + rename), so an interrupted save can't corrupt them.

## Logs

`log.ndjson` in the same directory. One JSON object per line. Tail it for live diagnostics:

```bash
tail -f ~/Library/Application\ Support/com.mpjhorner.IdeUltra/log.ndjson
```

The log includes cold-start timing, every workspace open, every file open and save, and any warnings (failed reload, save errors, missing tabs on restore).
