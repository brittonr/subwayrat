## Context

The subwayrat workspace contains 17 ratatui widget crates. Each handles its own internal layout (spreadsheet cells, node graphs, tree views), but nothing manages the spatial arrangement of multiple widgets as tiled windows within a terminal viewport.

Terminal applications that compose several widgets (editor + file tree + preview, or multiple spreadsheets side-by-side) currently require hand-rolled layout code. Niri's scrolling window manager demonstrates a clean model: windows live in an infinite horizontal strip of columns, and the viewport scrolls to follow focus. This translates directly to TUI — columns of stacked panels along a scrollable axis, with the visible region tracking whichever panel has focus.

We evaluated two external options:
- **Taffy** (CSS flexbox/grid engine): 8k+ lines of float-based CSS layout. Useful geometry primitives and constraint concepts, but the CSS box model is wrong for integer-cell TUI tiling.
- **iocraft** (React-like TUI framework): Uses taffy internally. Has a ScrollView component, but it replaces ratatui entirely and only scrolls a single content block vertically.

Neither fits. The algorithm we need is ~300 lines and ratatui-native.

## Goals / Non-Goals

**Goals:**
- Layout engine that arranges panels into a scrollable strip of columns, outputting ratatui `Rect` values
- Focus-driven viewport — the visible region follows the focused panel without manual scroll management
- Per-panel size constraints (fixed cells, proportional, content-measured)
- Column stacking — multiple panels within one column, stacked perpendicular to the scroll axis
- Runtime mutations — insert, remove, reorder, resize panels
- Keyboard navigation model across columns and within stacks
- Zero new external dependencies beyond ratatui

**Non-Goals:**
- Replacing ratatui's built-in `Layout` for simple splits — this is for multi-window tiling
- Animated scroll transitions — we compute positions, the consumer renders
- Floating/overlapping windows — all panels tile, no z-ordering
- Mouse-driven resize handles — resize is API-driven, consumers wire their own input
- CSS compatibility — no flexbox, no box model, no floats

## Decisions

### Decision: Integer cells, not floats
Use `u16` cell coordinates throughout, matching ratatui's `Rect`. No float→int conversion boundary. Taffy's pixel-rounding approach (cumulative rounding to avoid gaps) is unnecessary when you never leave integer space.

**Alternative**: Use taffy with flexbox-only features (like iocraft does). Rejected because the float→u16 conversion adds complexity for zero benefit in a cell-grid system, and flexbox carries 2500 lines of algorithm we don't need.

### Decision: Strip + Column + Window three-level hierarchy
```
Strip (scrollable axis)
├── Column 0
│   ├── Window A (full column height)
├── Column 1
│   ├── Window B (top half)
│   └── Window C (bottom half)
├── Column 2
│   └── Window D
```

The strip is the scrollable container. Columns partition the primary axis. Windows stack within columns along the cross axis. This matches niri's model and covers the common TUI patterns (sidebar + main + preview, multi-editor splits).

**Alternative**: Flat list of windows with automatic column assignment. Rejected because explicit column grouping gives the consumer control over which windows stack together.

### Decision: Constraint enum for sizing
```rust
enum SizeConstraint {
    Fixed(u16),          // exact cell count
    Proportion(f32),     // fraction of available space
    Min(u16),            // at least N cells
    MinMax(u16, u16),    // clamped range
}
```

Each window has width and height constraints. Column width is derived from the widest window's constraint. This is a stripped-down version of taffy's `AvailableSpace` concept without CSS baggage.

**Alternative**: Only fixed sizes. Rejected because proportional sizing is needed for "fill remaining space" patterns.

### Decision: Focus-driven viewport with centering bias
The viewport positions itself so the focused window is visible, with a centering bias — the focused window is brought toward the center of the viewport rather than snapped to an edge. When focus moves to an already-visible window, the viewport doesn't shift.

This matches niri's scroll behavior and avoids the jitter of always pinning focus to a viewport edge.

### Decision: Layout is a pure function
```rust
fn compute_layout(strip: &Strip, viewport_width: u16, viewport_height: u16) -> LayoutResult
```

No internal render state. No retained tree. The consumer owns the `Strip` data structure, calls `compute_layout`, gets back `Rect` positions for each visible window plus the viewport scroll offset. The consumer then renders using ratatui's standard `Frame::render_widget`.

**Alternative**: Retained layout tree with dirty tracking (like taffy's `TaffyTree`). Rejected because the strip layout is cheap enough to recompute every frame (linear in number of columns), and a retained tree adds complexity for caching that isn't needed at TUI scale (~10-50 windows).

### Decision: Separate crate, not a module in rat-widgets
The scrolling tiler is a layout engine, not a widget. It computes positions but doesn't own rendering. This is architecturally distinct from individual widgets in rat-widgets. A separate `rat-scrolltile` crate keeps the dependency graph clean — it depends on ratatui only.

## Risks / Trade-offs

**[Column count scaling]** → The linear scan is O(columns) per layout. At TUI scale (< 100 columns) this is sub-microsecond. If someone creates thousands of columns, performance degrades linearly. Mitigation: document the intended scale; if needed later, add a spatial index.

**[No float/overlap support]** → Tiling-only means no popup menus, tooltips, or modal dialogs can be expressed in this layout. Mitigation: these belong in a separate layering system (rat-layers already exists). The tiler handles spatial arrangement; overlays are orthogonal.

**[Proportional sizing depends on viewport]** → Windows with `Proportion` constraints change size when the terminal resizes. Consumers must handle content reflow. Mitigation: this is standard terminal behavior and consumers already handle resize via ratatui's event loop.

**[No animation primitives]** → The layout engine outputs static positions. Smooth scrolling would require the consumer to interpolate between frames. Mitigation: snap-to-position is the norm for TUI. Animation can be layered on top by the consumer if desired.
