## Why

The project has no layout engine for arranging multiple panels/windows within a terminal viewport. Existing crates handle widget-internal layout (grid columns, node graph positions) but nothing manages the top-level spatial relationship between windows. Applications that compose multiple rat-* widgets need a way to tile and scroll across them — the kind of arrangement niri does for Wayland windows, applied to TUI panels.

## What Changes

- New `rat-scrolltile` crate providing a scrolling tiled layout engine for ratatui widgets
- Strip-based layout: windows arranged in columns along a scrollable primary axis
- Column stacking: multiple windows stacked within a single column perpendicular to the scroll axis
- Focus-driven viewport: the visible region tracks the focused window automatically
- Per-window size constraints: fixed cell counts, proportional, or content-derived
- Insert, remove, reorder, and resize windows at runtime
- Keyboard navigation across the strip and within columns
- Zero new external dependencies — uses ratatui's `Rect` directly, integer cell coordinates throughout

## Capabilities

### New Capabilities
- `strip-layout`: Core layout algorithm that arranges columns along a primary axis and stacks windows within columns, computing cell-precise `Rect` positions
- `viewport-scroll`: Viewport tracking that follows focus, clips to the visible region, and exposes scroll offset for rendering
- `window-management`: Runtime operations for inserting, removing, reordering, and resizing windows in the strip
- `focus-navigation`: Keyboard-driven focus traversal across columns and within column stacks

### Modified Capabilities

## Impact

- New crate `crates/rat-scrolltile` added to workspace
- No changes to existing crates
- `showcase` crate will gain a demo page for the scrolling tiler
- Depends on ratatui (already a workspace dependency) — no new external deps
