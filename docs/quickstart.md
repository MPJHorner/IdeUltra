---
title: Quick Start
permalink: /quickstart.html
---

You can drive everything from the keyboard. Here's the 30-second tour.

## Open a folder

Press <code>⇧⌘O</code>, or click **File → Open Folder…**, and pick a directory. The file tree appears on the left.

The tree is **lazy**: only the directory you've expanded is read from disk. A workspace with 100,000 files opens instantly.

`.git`, `target`, `node_modules` are hidden by default, plus anything matched by the root `.gitignore`.

## Open a file

Click a file in the tree, or press <code>⌘O</code> for a native file picker. Multiple files become tabs above the editor.

- <code>⌘[</code> / <code>⌘]</code> — previous / next tab
- <code>⌘1</code>…<code>⌘9</code> — jump to tab N
- <code>⌘W</code> — close the active tab
- Middle-click a tab to close it

## Edit and save

Type. The window title and tab show a `●` while you have unsaved changes. <code>⌘S</code> writes back to disk and the dot disappears. You'll see "Saved …" in the bottom-right of the status bar.

If a file changes on disk while you have it open:

- **Clean buffer** — the editor reloads silently.
- **Dirty buffer** — an amber banner offers **Reload from disk** or **Keep mine**. Nothing happens until you choose.

## Find and replace

| | |
|-|-|
| <code>⌘F</code> | Open the find bar |
| <code>⌥⌘F</code> | Open find + replace |
| <code>Enter</code> / <code>⇧Enter</code> | Next / previous match |
| <code>Esc</code> | Close |

Three toggles in the bar:

- **Aa** — case-sensitive
- **W** — whole-word
- **.*** — treat the query as a regex

When regex mode is off, special characters like `.` and `(` are matched literally.

## Navigate

- <code>⌘P</code> — go to file (fuzzy finder). Type characters from the path —
  consecutive matches and word-boundary matches rank higher.
  ↑/↓ navigate, Enter opens, Esc closes.
- <code>⌘G</code> — go to line number
- <code>⌘B</code> — toggle the sidebar
- <code>⌘=</code> / <code>⌘-</code> / <code>⌘0</code> — zoom in / out / reset

## Quit and come back

Quit IdeUltra normally (<code>⌘Q</code>). When you relaunch, your folder, tabs, active file, theme, zoom factor and sidebar state are all restored — paths only, so any unsaved buffers since the last <code>⌘S</code> are not kept. Save your work before quitting.
