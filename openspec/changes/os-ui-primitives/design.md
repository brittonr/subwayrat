## Context

The subwayrat workspace contains focused ratatui widget crates. `rat-widgets` holds leaf-level controls (sliders, progress bars, text inputs). `rat-scrolltile` and `rat-layers` handle layout composition. Several spec'd capabilities (strip-layout, window-management, focus-navigation) address tiling window management but not the visual components that typically surround content in an OS-style interface.

Building a TUI that feels like an OS or IDE requires structural components — title bars, menus, toolbars, status bars, split panes — that wrap or frame child content. These don't exist in the workspace yet, and they follow different patterns from leaf widgets: they define regions for child content rather than rendering self-contained data.

The workspace uses Rust 2024 edition, MIT license, builder-pattern APIs, and the State/Style separation pattern from `rat-table` and `rat-tree`.

## Goals / Non-Goals

**Goals:**
- Structural UI components that frame or surround child content, following OS desktop conventions adapted for the terminal.
- Pure data models for each component, testable without a terminal.
- Builder-pattern configuration matching workspace conventions.
- Each component works standalone — no mandatory coupling between menu bar and title bar, for instance.
- Composable: dialog frame assembles title bar + content area + button row, but each part is usable independently.

**Non-Goals:**
- Mouse interaction. Keyboard-first for all interactive components; mouse can be layered on later.
- Focus management across components. Each component tracks its own internal focus (which menu item, which button). Cross-component focus routing is the consumer's job.
- Theming system. Components accept explicit `Style` values. A centralized theme registry is a separate concern.
- Animation or transition effects.
- Accessibility metadata (screen reader hints). Can be added later without API breakage.

## Decisions

### Separate crate (`rat-chrome`) instead of additions to `rat-widgets`

`rat-widgets` contains leaf controls that render their own content: a slider draws a track, a progress bar draws a fill. Chrome components are structural — they carve out regions for child content to render into. A title bar doesn't know what's below it. A split pane doesn't know what's in each half.

This distinction matters for dependency direction. Applications depend on `rat-chrome` for framing and on `rat-widgets` (or rat-table, rat-editor, etc.) for content. Keeping them separate avoids circular dependencies and keeps `rat-widgets` focused.

**Why not one big widget crate:** The workspace already demonstrated that separate crates scale better (rat-tree, rat-editor, rat-table each stand alone). Chrome components share enough internal structure (overlay rendering, child-region computation) to justify grouping, but they're architecturally different from leaf widgets.

### Model + Render split, no StatefulWidget

`rat-tree` uses ratatui's `StatefulWidget` trait. For chrome components, this trait is awkward because the widget needs to communicate sub-regions back to the caller — "here's where your content goes." `StatefulWidget::render` doesn't return anything.

Instead, each component provides:
- A model struct with state and mutation methods (e.g., `MenuBarModel`).
- A render function or struct that takes `&mut Frame`, a `Rect`, the model (by reference), and style config. Returns layout info (child rects, computed positions) so the caller knows where to render content.

```rust
// Example: SplitPane tells the caller where its two children go
let regions = split_pane.render(frame, area, &model, &style);
// regions.first -> Rect for left/top child
// regions.second -> Rect for right/bottom child
my_widget.render(frame, regions.first);
other_widget.render(frame, regions.second);
```

**Why not StatefulWidget:** The whole point of chrome is to produce child areas. `StatefulWidget::render(&self, area: Rect, buf: &mut Buffer, state: &mut S)` gives back nothing. We'd have to stash rects in the state, which is backwards — layout is an output, not an input. Explicit return values are clearer.

### Menu item model shared between MenuBar and ContextMenu

Both menu bars and context menus display lists of items with labels, accelerator hints, enabled/disabled state, separators, and submenus. Rather than duplicating this, a shared `MenuItem` enum covers both:

```rust
pub enum MenuItem {
    Action { label: String, accel: Option<String>, enabled: bool },
    Submenu { label: String, children: Vec<MenuItem> },
    Separator,
}
```

`MenuBarModel` holds top-level labels plus their dropdown `Vec<MenuItem>`. `ContextMenuModel` holds a flat `Vec<MenuItem>` and a position. Navigation logic (up/down through items, into/out of submenus) lives in a shared `MenuNav` helper that both models use.

**Why shared:** Menu navigation (skip separators, skip disabled items, enter submenu on Right, leave on Left) is identical regardless of whether the menu was triggered from a bar or a right-click. One implementation, tested once.

### SplitPane tracks position as enum, not f64

```rust
pub enum DividerPos {
    Ratio(f64),       // 0.0 to 1.0, fraction of available space
    FromStart(u16),   // fixed cells from left/top edge
    FromEnd(u16),     // fixed cells from right/bottom edge
}
```

**Why not always a ratio:** Fixed pixel offsets are common in IDEs (sidebar is exactly 30 columns wide). Ratios are common for equal splits. Supporting both avoids consumers converting back and forth. `FromEnd` handles cases like "status panel is always 10 rows from the bottom."

### Tooltip repositioning is caller's responsibility for X, automatic for Y

The tooltip takes an anchor (x, y) and content. It repositions vertically if it would overflow the viewport (flips above the anchor). Horizontal positioning is passed through as-is because the caller already knows the cursor column.

**Why not fully automatic:** Horizontal context varies too much. A tooltip for a menu item should align to the item. A tooltip for a status bar segment should align to the segment. The caller already has this info. Vertical is more uniform — "below the anchor, or above if no room" — so that's worth automating.

### DialogFrame is a composition helper, not a new widget trait

`DialogFrame` is a function that renders a border, title bar, and button row, then returns the inner content `Rect`. It's not a separate widget struct with its own `render` impl — it's a helper that calls the title-bar and button-row rendering internally.

```rust
let content_area = dialog_frame(frame, viewport, &dialog_model, &dialog_style);
// render your dialog content into content_area
```

**Why a function:** Dialogs vary wildly in content. Some have a form, some have text, some have a list. Making `DialogFrame` a struct that wraps content via generics or closures adds complexity without benefit. A function that returns "here's where your stuff goes" is simpler and composes with anything.

## Risks / Trade-offs

- **[Overlay rendering]** Tooltips, context menus, and dialogs render on top of existing content. Ratatui's `Clear` widget + re-render handles this, but z-ordering across multiple overlays (tooltip on top of context menu on top of dialog) requires the caller to render in the right order. Mitigation: document rendering order expectations. A layer manager can be added later via `rat-layers`.
- **[No mouse in v1]** Menus and toolbars without mouse feel different from OS conventions. Mitigation: keyboard navigation covers all operations. The model API (select item by index, toggle by id) is mouse-ready — adding mouse just means mapping click coordinates to the right model call.
- **[Crate scope creep]** Nine components in one crate is a lot. Mitigation: each component is a separate module with its own model/render pair. If any component grows complex enough to justify extraction (menu system is the most likely candidate), it can become its own crate later without breaking the public API — just re-export from `rat-chrome`.
- **[Child region coordination]** Chrome components return `Rect` values that callers must use correctly. If a caller ignores the returned content rect and renders into the full area, content will overlap the chrome. Mitigation: examples and doc comments showing the pattern. This is the same contract ratatui's `Block::inner()` already establishes.
