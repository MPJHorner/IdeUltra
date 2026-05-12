# IdeUltra Style Guide

The design system that drives every pixel of IdeUltra. Distilled from
Zed, Linear (light), Vercel Geist, Stripe docs, iA Writer, JetBrains
Fleet, and shadcn/ui.

This document is **the source of truth** — `src/style.rs` and
`src/ui/components/` are direct expressions of it. When you're tempted
to pick a stray hex, padding, or radius in a UI file, come back here.

---

## Aesthetic direction: "luxury light"

IdeUltra is a code editor for **2026**. The visual answer to *"what
does a code editor look like when it stops trying to look like a 2010
code editor?"* is:

* **White or near-white paper. No backdrops of indigo or purple.** The
  editor canvas is `#FFFFFF`. The sidebar and status bar are one tiny
  step away from white. The eye lands on the code, not the chrome.
* **Type does the heavy lifting.** Hierarchy comes from size and
  weight, not from coloured pills or bright accents.
* **The accent is deep ink.** Looks black against white, has just
  enough blue-grey cast to feel "designed". Used only for primary
  buttons, focus rings, and the "selected" state. Never for decoration.
* **Status colours are deep, never neon.** Crimson, amber, forest,
  navy. No 60s neon-lime success badges.
* **Borders barely exist** (~6 % alpha black). Surfaces separate by
  whitespace, not by lines.
* **Shadows are almost imperceptible** in light mode. We trust the
  border + the spacing.
* **Generous whitespace.** Density is intentionally lower than VS Code
  or IntelliJ. We can afford it because we're not trying to cram in a
  language server.

Dark mode is **not** the same product with a hue inversion. It's a
parallel charcoal / zinc complement, equally restrained. Light is the
hero; dark is the night shift.

**Reference points to keep on your reading list:**

* Linear's light theme — proof that "white + grey" can be premium.
* Vercel Geist colors / Geist typography — the gold standard for dev-
  tool restraint.
* Stripe docs — the canonical "white paper with a tiny accent" look.
* iA Writer — typography as the only design.
* GitHub light theme — what we want, but quieter.

---

## 0. Principles

These are the seven rules the research surfaced across *every* modern
IDE/dev-tool UI. Internalise them; everything else is mechanics.

1. **Perceptually-uniform palettes, generated not picked.** Each surface
   layer is a fixed lightness step from the canvas. Theme switching and
   tints "just work" because every step is calibrated.
2. **Tokens come in pairs: surface ↔ foreground.** Every background has
   a sibling text color. You never apply text color independent of the
   surface it sits on.
3. **Every interactive element has 4 states: default → hover → active →
   selected/disabled.** Each state is a fixed lightness or alpha bump
   above the previous. Naming is consistent.
4. **The type scale is tight: 11 / 12 / 13 / 14 / 16 / 20 / 24.** Dev
   chrome lives in the 12-14 band. No "60-step Material scale".
5. **Restraint with accent color.** One accent, used only for focus,
   primary CTAs, selection, links. Status colors are a separate channel
   used sparingly. Chromatic noise comes from icons, not chrome.
6. **Borders are 1 px and barely visible. Big shadows are reserved for
   floating surfaces** (modals, popovers). Inline UI gets a subtle
   border + tiny shadow; modals get the dramatic stack.
7. **Motion is fast, ease-out, never gratuitous.** Hovers ≤ 120 ms,
   modals ≤ 200 ms. Nothing bounces.

---

## 1. Code-editor specific rules

These overlay the universal principles:

1. **The editor canvas is the *darkest* surface in the dark theme.**
   Chrome (sidebar, tabs, status bar) is brighter so the eye lands on
   code. This is inverted from most web apps.
2. **Active line is alpha tint, not opaque background.** ~75 % alpha of
   the next-darker surface over the editor bg. Subtle enough not to
   fight syntax colors.
3. **Status colors come as 3-variant bundles:** foreground +
   `~10 % alpha` background pill + matching desaturated border. Used for
   diagnostics and diff gutters. Opaque red destroys readability — alpha
   is the trick.
4. **VCS colors are distinct from semantic status.** Added/modified/
   deleted have their own greens/yellows/reds tuned for diff gutters and
   word-level highlights, not the same as success/warning/error.
5. **Mono font is a first-class typography choice.** Used for code,
   inline `tokens`, keycap labels, line/col numbers, commit SHAs.

---

## 2. Design tokens

All token values are dark-theme first. Light-theme is the inverted
sibling — see §6.

### 2.1 Colors

Two token taxonomies wired together:

**Surface ramp** — each step is a perceptually consistent lightness
bump above the previous. Light is the hero.

| Token            | Light value          | Dark value           | Use |
|------------------|----------------------|----------------------|-----|
| `bg.canvas`      | `#FFFFFF`            | `#0B0B0D`            | Editor body (lightest light / darkest dark) |
| `bg.chrome`      | `#FBFBFC`            | `#101012`            | Sidebar, status bar |
| `bg.surface`     | `#F5F5F7`            | `#161619`            | Tab strip, find bar |
| `bg.elevated`    | `#FFFFFF`            | `#1B1B1E`            | Popovers, dropdowns, menus |
| `bg.modal`       | `#FFFFFF`            | `#1E1E22`            | Modals, command palette |
| `bg.subtle`      | `rgba(0,0,0,0.02)`   | `rgba(255,255,255,0.03)` | Empty-state plates, hover backgrounds |

**Foreground ramp** — semantic text colors. Never pure black on white
(too harsh); deep ink at `#0A0A0A`.

| Token             | Light value | Dark value | Use |
|-------------------|-------------|------------|-----|
| `text.primary`    | `#0A0A0A`   | `#F0F0F2`  | Body, labels, headings |
| `text.secondary`  | `#444448`   | `#AEAEB2`  | Secondary copy, subtitles |
| `text.muted`      | `#888892`   | `#727276`  | Captions, dim path hints, placeholder |
| `text.disabled`   | `#C0C0C6`   | `#48484C`  | Disabled UI |
| `text.on-accent`  | `#FFFFFF`   | `#0B0B0D`  | Text on the accent fill |

**Border ramp** — three visibility tiers; almost invisible by design.

| Token             | Light                  | Dark                       |
|-------------------|------------------------|----------------------------|
| `border.subtle`   | `rgba(0,0,0,0.04)`     | `rgba(255,255,255,0.07)`   |
| `border.default`  | `rgba(0,0,0,0.07)`     | `rgba(255,255,255,0.12)`   |
| `border.strong`   | `rgba(0,0,0,0.12)`     | `rgba(255,255,255,0.23)`   |

**Accent** — deep ink in light; ivory in dark. *Looks like text*, not
like a brand color.

| Token             | Light      | Dark       |
|-------------------|------------|------------|
| `accent`          | `#18181B`  | `#F4F4F6`  |
| `accent.hover`    | `#27272A`  | `#FFFFFF`  |
| `accent.bg`       | `rgba(0,0,0,0.03)` | `rgba(255,255,255,0.11)` |
| `accent.border`   | `rgba(0,0,0,0.11)` | `rgba(255,255,255,0.31)` |

**Semantic status** — *each* status color ships in **3 variants** so
diagnostic pills don't shout. **Deep, never neon.** These values are
the LIGHT-theme set; dark uses softer pastels of the same hues.

| Status   | Foreground | Background (~7 %α) | Border (~20 %α) |
|----------|------------|---------------------|-----------------|
| Error    | `#B91C1C`  | crimson 7 %         | crimson 20 %    |
| Warning  | `#B45309`  | amber 7 %           | amber 20 %      |
| Success  | `#15803D`  | forest 7 %          | forest 20 %     |
| Info     | `#1D4ED8`  | navy 7 %            | navy 20 %       |

**VCS** — distinct from semantic. Tuned for code gutters.

| State    | Foreground | Background (15 %α) |
|----------|------------|---------------------|
| Added    | `#27A657`  | `rgba(39,166,87,0.15)` |
| Modified | `#D3B020`  | `rgba(211,176,32,0.15)` |
| Deleted  | `#E06C76`  | `rgba(224,108,118,0.15)` |
| Renamed  | `#B083DA`  | `rgba(176,131,218,0.15)` |
| Untracked| `#5EB1F0`  | `rgba(94,177,240,0.15)` |

### 2.2 Interactive states (4-state lifecycle)

Every interactive element walks this lifecycle. The state tints are
*absolute* — they don't depend on the underlying surface.

| State        | Bg tint                          | Border |
|--------------|----------------------------------|--------|
| default      | (transparent, or `bg.elevated`)  | `border.default` (only on filled bg) |
| hover        | `rgba(255,255,255,0.04)`         | `border.default` |
| active       | `rgba(255,255,255,0.08)`         | `border.strong` |
| selected     | `accent.bg`                      | `accent.border` |
| disabled     | (transparent)                    | (none); `text.disabled` foreground |
| focus-ring   | adds `1.5 px accent` outline atop any state | |

**Hover is never an outline.** Only background tint or color shift.
Outline is reserved for keyboard-focus rings.

### 2.3 Spacing — 4 px grid

| Token     | px |
|-----------|----|
| `space.1` | 4  |
| `space.2` | 8  |
| `space.3` | 12 |
| `space.4` | 16 |
| `space.5` | 20 |
| `space.6` | 24 |
| `space.8` | 32 |
| `space.10`| 40 |
| `space.12`| 48 |
| `space.16`| 64 |

**Conventions:**
- 6 px between an icon and its label
- 8 px intra-component (button padding-y, list row inner)
- 12 px between rows
- 16 px panel padding
- 24 px between sections inside a modal
- 48 px between major regions

### 2.4 Radii

| Token        | px | Use |
|--------------|----|-----|
| `radius.xs`  | 4  | inline pills, keycaps |
| `radius.sm`  | 6  | buttons, inputs, list rows |
| `radius.md`  | 8  | cards, secondary panels |
| `radius.lg`  | 10 | modals, command palette (anchor) |
| `radius.xl`  | 14 | large dialogs |

### 2.5 Typography

| Role            | Family    | px | Weight | LH  | Tracking |
|-----------------|-----------|----|--------|-----|----------|
| caption         | sans      | 11 | 500    | 14  | +0.02 em (uppercase) |
| label-sm        | sans      | 12 | 500    | 16  | 0 |
| label / UI      | sans      | 13 | 500    | 18  | 0 |
| body            | sans      | 14 | 400    | 20  | 0 |
| body-lg         | sans      | 16 | 400    | 24  | 0 |
| heading-sm      | sans      | 18 | 600    | 24  | -0.005 em |
| heading-md      | sans      | 22 | 600    | 28  | -0.01 em |
| heading-lg      | sans      | 28 | 700    | 34  | -0.015 em |
| mono-code       | mono      | 13.5 | 400  | 20  | 0 |
| mono-ui         | mono      | 12 | 500    | 16  | 0 |
| keycap          | mono      | 11 | 500    | 14  | 0 |

**Conventions:**
- App chrome (sidebar items, tabs, status bar, find bar) → label/UI 13 px
- Modal body text → body 14 px
- Section headers inside modals → heading-sm
- File paths, identifiers, line numbers, commit SHAs → mono-ui
- Keymap labels → keycap, in a 4 px radius keycap pill

Tabular numerals on metrics (line counts, byte sizes) when supported.

### 2.6 Shadows / Elevation

Three tiers. **Always pair shadow with a subtle 1 px top highlight on
dark.**

| Token       | Spec                                                                                                          |
|-------------|---------------------------------------------------------------------------------------------------------------|
| `shadow.sm` | `0 1px 2px rgba(0,0,0,0.4)`                                                                                   |
| `shadow.md` | `0 4px 12px rgba(0,0,0,0.35), 0 1px 2px rgba(0,0,0,0.4)`                                                       |
| `shadow.lg` | `0 16px 40px rgba(0,0,0,0.45), 0 4px 12px rgba(0,0,0,0.35), inset 0 1px 0 rgba(255,255,255,0.04)`             |

- `sm` — buttons, inputs, list rows (when needed)
- `md` — popovers, dropdowns, sidebar menus, banners
- `lg` — modals, command palette, finder, preferences window

### 2.7 Motion

| Use                                  | Duration | Easing                           |
|--------------------------------------|----------|----------------------------------|
| Hover, focus tint                    | 100 ms   | ease-out                         |
| Active press                         | 80 ms    | ease-out                         |
| Panel / popover open                 | 160 ms   | cubic-bezier(0.16, 1, 0.3, 1)    |
| Modal / dialog open                  | 200 ms   | ease-out + scale 0.98 → 1.0      |
| Layout shifts (split toggle, etc.)   | 240 ms   | spring (egui's natural defaults) |

Never animate over 300 ms for interaction feedback. egui doesn't have a
proper animation system — most of this is achieved by relying on
egui's default frame-by-frame smoothing. The numbers serve as
*intentional restraint* when we do reach for `ctx.animate_value_with_time`.

### 2.8 Iconography

- 16 px standard, 14 px dense, 20 px header / hero
- Outline icons by default at 1.5 px stroke
- Filled icons reserved for selected/active state
- Monochromatic in chrome (inherit `text.secondary` or `text.muted`)
- Colorful permitted only for status indicators and integration logos
  (Raycast pattern)

Today IdeUltra uses Unicode glyphs (🦀, ⚙, ⚡) — fine for v1; replace
with an SVG icon set in v2.

---

## 3. Component recipes

Every component lives in `src/ui/components/` and is exported through
`src/ui/components/mod.rs`. The recipes below define each component's
contract.

### 3.1 `Card` — visual container

A rounded surface with optional border + shadow. The base building
block for everything that isn't a button.

- `bg.surface` background
- `border.subtle` 1 px border
- `radius.md` rounding
- Padding configurable (default `space.4` = 16 px)
- No hover state — it's a container, not interactive

Useful for sidebar group containers, settings sections, find bar.

### 3.2 `ModalFrame` — floating-surface wrapper

A `Card` with extra ceremony: rounded corners (`radius.lg`), `shadow.lg`,
top inset highlight, no border (the shadow does the work).

Used by every modal: finder, command palette, preferences, keymap
picker, name prompt, delete confirm, close confirm, replace confirm,
diff modal, recovery modal.

Standard contents:
1. Title row (sometimes hidden — finder/command palette use a search
   input as the title equivalent)
2. Body
3. Optional action row at the bottom: hint label left, buttons right

### 3.3 `PrimaryButton` / `GhostButton` / `IconButton`

- `PrimaryButton` — gradient fill of `accent → accent.hover`, `text.on-accent`,
  `radius.sm`, padding `space.4`/`space.2`, weight 600, label/UI 13 px.
- `GhostButton` — transparent fill, `text.primary`, hover → `bg.subtle`,
  `radius.sm`, padding `space.4`/`space.2`. Border only on hover.
- `IconButton` — 32 × 32 square, transparent, hover → `bg.subtle`,
  `radius.sm`. The topbar GitHub + theme buttons.

All three accept a `disabled` state.

### 3.4 `ListRow` — selectable list item

Used by the finder modal, command palette, project search results,
keymap picker entries, MRU lists.

- 32 px default height (40 px when accessory keymap label is shown)
- Inset `space.1` (4 px) horizontally so the selected pill floats
- `radius.sm` (6 px)
- States: default → hover (`bg.subtle`) → selected (`accent.bg`,
  `accent.border`, `text.on-accent` for fg)
- Content: `[icon] [primary text] … [secondary text] [hint keys]`

### 3.5 `Section` / `SectionHeader`

Vertical grouping inside a modal or panel.

- `SectionHeader` — caption-style label (11 px uppercase, +0.04 em
  letter-spacing, `text.muted`), bottom margin `space.2`
- Section body — vertical stack with `space.3` between children

### 3.6 `EmptyState`

The trick that makes panels feel deliberate.

- 56–64 px icon (`text.muted`) at optical center of panel
- 8 px gap
- 14 px medium-weight title
- 8 px gap
- 13 px regular-weight one-line description, `text.muted`
- Optional 16 px gap + secondary action
- 40 px breathing room above and below the stack

### 3.7 `Pill` / `Badge`

Small chromatic tag. Two flavours:

- `Pill::status(kind)` — uses the 3-variant status bundle (e.g.
  Modified, Deleted) for git gutters, diagnostic chips.
- `Pill::neutral(label)` — `bg.subtle` background, `text.secondary`,
  `radius.xs`, 11 px caption type. Used for "In selection", "Untitled",
  workspace count, etc.

### 3.8 `Hint` — keymap-cheat-sheet label

The "↑↓ navigate · ⏎ open · esc close" footer in modals.

- Caption type (11 px, `text.muted`)
- Keycaps rendered with `mono-ui` 11 px in a `~3 px` radius pill, joined
  by middle dot (·)

### 3.9 `SearchInput`

A frameless `TextEdit` with a leading glyph (⌕ / ⌘ / etc.) at the
modal top. Wide, 15 px proportional font, prominent.

### 3.10 `Banner`

Top-of-window strip (e.g. the update notifier). Background `bg.subtle`,
no shadow, 1 px border-bottom in `border.subtle`. Two slots: left-aligned
text, right-aligned action(s).

---

## 4. egui idioms and gotchas

`egui` 0.28 is our painting layer. Quirks worth knowing:

1. **`ctx.set_visuals(...)`** replaces the entire visuals struct —
   call once per frame, don't try to mutate fields between frames
   (they're reset on each `set_visuals`).
2. **`ctx.style_mut(|s| ...)`** is the way to set spacing, scroll-bar
   width, item-spacing, etc. Call once per frame too.
3. **`Frame::window(&ctx.style())`** gives you the default modal frame.
   You almost always want to override fill, stroke, rounding, shadow.
4. **`egui::Color32::from_rgba_premultiplied`** is the right constructor
   for transparent tints; `from_rgba_unmultiplied` is rarely correct.
5. **Shadows are an `egui::epaint::Shadow { offset, blur, spread,
   color }`.** No CSS-style multi-stop shadows — combine via background
   layering if you need depth.
6. **`Frame::default()` does NOT carry the visuals' panel-fill** — it's
   fully transparent unless you set `.fill(...)`.
7. **`ui.painter().rect_stroke(...)`** is the way to draw an outline-only
   rect (e.g. focus ring) without affecting layout. Use `painter_at` to
   clip to a parent rect.
8. **`Sense::click_and_drag()`** is required for drag detection on
   anything that isn't a built-in draggable widget. `response.dragged()`
   / `drag_started()` / `drag_stopped()` come from there.
9. **`response.context_menu(|ui| ...)`** is right-click menu support
   built in — no need to roll your own.
10. **`ScrollArea::vertical().auto_shrink([false, false])`** is almost
    always what you want — the default shrinks to content and looks
    janky in modals.
11. **Layout reads bottom-to-top in `right_to_left` mode** — confusing
    until you get it. Used for "action buttons right, hint label left"
    rows.
12. **egui's `RichText` accumulates** — chain `.size(13.0).strong()
    .color(c)` on a single `RichText::new(...)`. Don't wrap labels in
    multiple text builders.
13. **Hover/click on `Frame::show`** — the returned `InnerResponse`'s
    `.response` is the frame's hit-rect, but `.interact(Sense::click())`
    is needed to make it clickable; `Sense::hover()` for hover-state.
14. **`Id::new(("scoped", path))`** — when you need multiple instances of
    the same widget (e.g. tabs, list rows), seed the id with a tuple
    including the unique key (path, index, etc.). Prevents id clashes.
15. **TextEditState** lives in `ctx.memory()` and survives across frames.
    Use `TextEditState::load(ctx, id)` to read the caret and
    `.store(ctx, id)` to put it back.

---

## 5. Apply checklist

When updating a UI file:

- [ ] Replace hex literals with `tokens::*` from `src/style.rs`
- [ ] Replace `egui::Frame::default()` with `components::Card::new(...)`
- [ ] Replace inline `egui::Window::new(...)` with `components::ModalFrame::show(...)`
- [ ] Replace buttons with `PrimaryButton` / `GhostButton` / `IconButton`
- [ ] Replace list rows with `ListRow`
- [ ] Replace "empty" branches with `EmptyState`
- [ ] Verify spacing matches the 4 px grid
- [ ] Verify text sizes match the type scale
- [ ] Add the keymap hint footer where appropriate
- [ ] Check both dark and light themes

---

## 6. Light-theme inversion

Light theme follows the same token names but inverts the ramp. Specific
overrides in §2.1. Tints (`accent.bg`, `status.*.bg`, hover tints) drop
their alpha by ~30 % in light mode because pure dark on light is
otherwise too aggressive.

When designing, **start dark** — dark is the editor's primary skin —
and verify light is acceptable, not the other way round.

---

## 7. Reference

The research that produced this document is in
[`docs/design-research.md`](docs/design-research.md). Read it once if
you're going to make a structural change. The TL;DR is in §0.
